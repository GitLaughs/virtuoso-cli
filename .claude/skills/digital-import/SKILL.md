---
name: digital-import
description: |
  Import P&R (Genus + Innovus) products into Virtuoso: GDS layout, Verilog
  schematic/symbol, power labels, and label restyling. Four-step pipeline driven
  entirely from vcli skill exec — no Python bridge or GUI required.

  Use when: (1) user wants to pull a routed GDS or post-P&R netlist into Virtuoso,
  (2) layout labels look giant/unreadable after import, (3) user mentions strmin /
  ihdl / digital import / P&R-to-schematic flow.
allowed-tools:
  - Bash(vcli *)
argument-hint: "[step, e.g. 'import GDS' or 'restyle labels for DIG_OUTPUT/LFSR_32BIT']"
---

# Digital Import — 4-Step Pipeline

| # | Step | Tool | Output |
|---|------|------|--------|
| 1 | **import_gds** | `strmin` | `layout` view |
| 2 | **import_verilog** | `ihdl` | `schematic` + `symbol` |
| 3 | **add_power_labels** | SKILL `dbCreateLabel` | VDD/VSS labels on M1.pin |
| 4 | **restyle_labels** | SKILL `dbSetq` via `~>attr` | Shrink `text/drawing` to 0.05 µm; bump `<layer>/pin` to 0.2 µm + roman |

Steps 3 and 4 are pure SKILL — run directly from `vcli skill exec`.
Steps 1–2 invoke Cadence batch tools via SKILL `system()`.

## Prerequisite — cds.lib DEFINE lines

`strmin` and `ihdl` create cellview directories on disk but **do NOT edit cds.lib**.
If a library has no `DEFINE` line, Virtuoso won't see the result. Add before running:

```
DEFINE DIG_OUTPUT      /home/you/work/DIG_OUTPUT
DEFINE tsmcN28         /cad/process/.../tsmcN28
DEFINE tcbn28hpcplus   /cad/process/.../bwp12t30p140
```

---

## Step 1 — Import GDS (strmin)

```bash
# Standard: provide a ref-libs directory
vcli skill exec 'system("strmin -library DIG_OUTPUT \
  -strmFile /path/to/foo.route_tapeout.gds \
  -techLib tsmcN28 \
  -refLibList /path/to/ref_libs_dir \
  -logFile /tmp/strmin.log")'

# Shortcut: if cds.lib already DEFINEs every referenced lib
# Pass the magic literal XST_CDS_LIB — strmin consults the CWD cds.lib instead
vcli skill exec 'system("strmin -library DIG_OUTPUT \
  -strmFile /path/to/foo.route_tapeout.gds \
  -techLib tsmcN28 \
  -refLibList XST_CDS_LIB \
  -logFile /tmp/strmin.log")'

# Verify
vcli skill exec 'dbOpenCellViewByType("DIG_OUTPUT" "LFSR_32BIT" "layout" nil "r")~>bBox'
```

`XST_CDS_LIB` is a strmin magic literal — mutually exclusive with a real ref file.
Use it when the project's cds.lib is already curated (every dependency has a DEFINE).

---

## Step 2 — Import Verilog (ihdl)

```bash
vcli skill exec 'let((f)
  f = outfile("/tmp/ihdl_param" "w")
  fprintf(f "reference_libraries := tcbn28hpcplusbwp12t30p140\n")
  fprintf(f "design_library := DIG_OUTPUT\n")
  fprintf(f "input_file := /path/to/foo_import.v\n")
  fprintf(f "structural_views := 5\n")   ; schematic + functional (IC618 encoding)
  close(f)
  system("ihdl -ihdlFile /tmp/ihdl_param -log /tmp/ihdl.log"))'
```

If ihdl fails, check `<virtuoso_workdir>/verilogIn.batch.log` for diagnostics.

---

## Step 3 — Add Power Labels

Walk instances to find one with VDD/VSS terminals, read pin geometry, transform
through instance xform, drop labels at the layout midline.

```bash
vcli skill exec 'let((cv vddY vssY midX)
  cv = dbOpenCellViewByType("DIG_OUTPUT" "LFSR_32BIT" "layout" nil "a")
  ; find VDD/VSS Y coords from instance pin geometry + xform (simplified)
  ; then place labels
  dbCreateLabel(cv (list "M1" "pin") (list midX vddY) "VDD!" "centerLeft" "R0" "roman" 1.0)
  dbCreateLabel(cv (list "M1" "pin") (list midX vssY) "VSS!" "centerLeft" "R0" "roman" 1.0)
  dbSave(cv)
  dbClose(cv))'
```

TSMC defaults: layer `M1`, purpose `pin`, font `roman`, height `1.0`.
Sky130: use `VPWR`/`VGND`, height `0.4` (5T row ≈ 0.5 µm).

---

## Step 4 — Restyle Labels

Innovus stamps hundreds of `text/drawing` labels at 1 µm — taller than the cells
themselves on a tiny digital block. This single SKILL traversal fixes both classes:

```bash
# Quick version (no floorplan, bbox heuristic for pin orientation)
vcli skill exec 'let((cv bb xmin ymin xmax ymax thr n_text n_pin n_or pin_shapes)
  cv = dbOpenCellViewByType("DIG_OUTPUT" "LFSR_32BIT" "layout" nil "a")
  thr = 4.0   ; distance threshold (µm) to edge
  n_text = 0  n_pin = 0  n_or = 0
  pin_shapes = nil
  ; Pass A: set heights/font — read bBox AFTER this pass (see gotcha below)
  foreach(s cv~>shapes
    when(s~>objType == "label"
      cond(
        (s~>layerName == "text" && s~>purpose == "drawing"
           s~>height = 0.05
           n_text = n_text + 1)
        (s~>purpose == "pin"
           s~>height = 0.2
           s~>font = "roman"
           s~>justify = "centerLeft"
           n_pin = n_pin + 1
           pin_shapes = cons(s pin_shapes))
      )
    )
  )
  ; Pass B: orient pin labels by bbox distance (read bBox AFTER pass A)
  bb = cv~>bBox
  xmin = caar(bb)  ymin = cadar(bb)
  xmax = caadr(bb) ymax = cadadr(bb)
  foreach(s pin_shapes
    let((x y dl dr db dt mn)
      x = car(s~>xy)  y = cadr(s~>xy)
      dl = x - xmin  dr = xmax - x
      db = y - ymin  dt = ymax - y
      mn = min(min(dl dr) min(db dt))
      when(mn < thr
        cond(
          (mn == db  s~>orient = "R270"  n_or = n_or + 1)
          (mn == dt  s~>orient = "R90"   n_or = n_or + 1)
          (mn == dl  s~>orient = "R180"  n_or = n_or + 1)
          (mn == dr  s~>orient = "R0"    n_or = n_or + 1)
        )
      )
    )
  )
  dbSave(cv)
  dbClose(cv)
  sprintf(nil "text/drawing: %d -> 0.05 | pin: %d -> 0.2 roman | oriented: %d"
              n_text n_pin n_or))'
```

Expected output: `text/drawing: 505 -> 0.05 | pin: 30 -> 0.2 roman | oriented: 28`

---

## Critical SKILL Gotchas

### `~>attr = val` not `dbSet` for label properties

```skill
; WRONG — silently no-ops on label height in IC618/IC23
dbSet(s 'height 0.05)

; CORRECT — compiles to dbSetq, actually works
s~>height = 0.05
s~>font   = "roman"
s~>orient = "R90"
```

This applies to `height`, `font`, `justify`, `orient` on label shapes.
`dbSet` returns `t` with no error but the value doesn't change.

### Two-pass approach for pin orientation (bbox timing bug)

If you read `cv~>bBox` BEFORE resizing `text/drawing` labels, the bbox is inflated
by 1 µm labels — corner pins appear farther from the edge than they are and get
missed by the `mn < thr` check. Always:

1. **Pass A**: set heights on all labels (bBox still wrong)
2. **Pass B**: read `cv~>bBox`, then classify pin edges — bBox now reflects final sizes

Alternatively, parse `editPin -side X` from the Innovus floorplan Tcl for source-of-truth
orientation (bypasses the heuristic entirely):
- `Top` → `R90`, `Bottom` → `R270`, `Left` → `R180`, `Right` → `R0`
- `-edge N` (integer form) is Innovus-version-dependent — skip it, fall back to bbox

### strmin does not update cds.lib

Always add `DEFINE` lines to cds.lib before running strmin. Missing a lib → Virtuoso
silently ignores the imported cells (no error from strmin itself).

### Bus bracket rewriting

strmin with `-replaceBusBitChar` rewrites `signal[3]` → `signal<3>` in labels.
When matching pin names from Innovus floorplan Tcl against imported labels, replace
`[` → `<` and `]` → `>`.

---

## PDK Portability

| Flag | Step | TSMC N28 default | Sky130 override |
|------|------|-----------------|-----------------|
| tech library | 1 | `tsmcN28` | `sky130A` |
| ref library | 2 | `tcbn28hpcplusbwp12t30p140` | `sky130_fd_sc_hd` |
| power/ground pin names | 3 | `VDD` / `VSS` | `VPWR` / `VGND` |
| label height | 3 | `1.0` µm | `0.3–0.4` µm (5T row ≈ 0.5 µm) |

`ihdl structural_views := 5` is the IC618 SP201 encoding for schematic + functional.
On a different IC release, check the *Verilog In for Virtuoso Design Environment User Guide*
for the correct integer.
