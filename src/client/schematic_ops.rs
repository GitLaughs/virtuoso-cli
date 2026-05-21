use crate::client::bridge::escape_skill_string;

/// SKILL global that holds the currently active schematic cellview.
///
/// # Concurrency note
/// `RB_SCH_CV` is a process-wide SKILL global. Concurrent vcli processes
/// that call `open_cellview` on *different* cellviews will overwrite each
/// other's global. For serial scripting this is safe; for parallel use,
/// callers must serialize schematic commands or use separate Virtuoso sessions.
const SCH_CV_VAR: &str = "RB_SCH_CV";

/// SKILL guard: checks that the cellview global is bound and still valid,
/// errors with a helpful message otherwise.
fn cv_guard() -> String {
    format!(
        r#"unless(boundp('{SCH_CV_VAR}) && {SCH_CV_VAR} error("RB_SCH_CV is not set - run 'vcli schematic open lib/cell/view' first"))"#
    )
}

#[derive(Default)]
pub struct SchematicOps;

impl SchematicOps {
    pub fn new() -> Self {
        Self
    }

    pub fn create_instance(
        &self,
        lib: &str,
        cell: &str,
        view: &str,
        name: &str,
        origin: (i64, i64),
        orient: &str,
    ) -> String {
        let lib = escape_skill_string(lib);
        let cell = escape_skill_string(cell);
        let view = escape_skill_string(view);
        let name = escape_skill_string(name);
        let orient = escape_skill_string(orient);
        let (x, y) = origin;
        let guard = cv_guard();
        format!(
            r#"let((cv master inst) {guard} cv = RB_SCH_CV master = dbOpenCellViewByType("{lib}" "{cell}" "{view}" nil "r") inst = dbCreateInst(cv master "{name}" list({x} {y}) "{orient}" 1) inst)"#
        )
    }

    pub fn create_wire(&self, points: &[(i64, i64)], layer: &str, net_name: &str) -> String {
        let layer = escape_skill_string(layer);
        let net_name = escape_skill_string(net_name);
        let pts: String = points
            .iter()
            .map(|(x, y)| format!("list({x} {y})"))
            .collect::<Vec<_>>()
            .join(" ");
        let guard = cv_guard();
        format!(
            r#"let((cv) {guard} cv = RB_SCH_CV dbCreateWire(cv dbMakeNet(cv "{net_name}") dbFindLayerByName(cv "{layer}") list({pts})))"#
        )
    }

    #[allow(dead_code)]
    pub fn create_wire_between_terms(
        &self,
        inst1: &str,
        _term1: &str,
        inst2: &str,
        _term2: &str,
        net_name: &str,
    ) -> String {
        let inst1 = escape_skill_string(inst1);
        let inst2 = escape_skill_string(inst2);
        let net_name = escape_skill_string(net_name);
        let guard = cv_guard();
        format!(
            r#"let((cv net) {guard} cv = RB_SCH_CV net = dbMakeNet(cv "{net_name}") dbCreateWire(net dbFindTermByName(cv "{inst1}") dbFindTermByName(cv "{inst2}")))"#
        )
    }

    pub fn create_wire_label(&self, net_name: &str, origin: (i64, i64)) -> String {
        let net_name = escape_skill_string(net_name);
        let (x, y) = origin;
        let guard = cv_guard();
        format!(
            r#"let((cv net) {guard} cv = RB_SCH_CV net = dbFindNetByName(cv "{net_name}") when(net dbCreateLabel(cv net "{net_name}" list({x} {y}) "centerCenter" "R0" "stick" 0.0625)))"#
        )
    }

    pub fn create_pin(&self, net_name: &str, pin_type: &str, origin: (i64, i64)) -> String {
        let net_name = escape_skill_string(net_name);
        let (pin_master, direction) = match pin_type.to_ascii_lowercase().as_str() {
            "input" | "in" | "ipin" => ("ipin", "input"),
            "output" | "out" | "opin" => ("opin", "output"),
            "inout" | "io" | "iopin" | "inputoutput" => ("iopin", "inputOutput"),
            _ => ("ipin", "input"),
        };
        let pin_master = escape_skill_string(pin_master);
        let direction = escape_skill_string(direction);
        let (x, y) = origin;
        let guard = cv_guard();
        format!(
            r#"let((cv master pinInst stubEnd wireObjs wireObj) {guard} cv = RB_SCH_CV master = dbOpenCellViewByType("basic" "{pin_master}" "symbol" nil "r") pinInst = schCreatePin(cv master "{net_name}" "{direction}" "{net_name}" list({x} {y}) "R0") stubEnd = list({x} + 0.45 {y}) wireObjs = schCreateWire(cv "route" "full" list(list({x} {y}) stubEnd) 0 0 0 nil nil) wireObj = car(wireObjs) schCreateWireLabel(cv wireObj stubEnd "{net_name}" "lowerCenter" "R0" "stick" 0.0625 nil) pinInst)"#
        )
    }

    pub fn clear_current_cellview(&self) -> String {
        let guard = cv_guard();
        format!(
            r#"let((cv) {guard} cv = RB_SCH_CV foreach(inst cv~>instances dbDeleteObject(inst)) foreach(shape cv~>shapes dbDeleteObject(shape)) foreach(term cv~>terminals dbDeleteObject(term)) foreach(net cv~>nets dbDeleteObject(net)) t)"#
        )
    }

    pub fn check(&self) -> String {
        let guard = cv_guard();
        format!(r#"let((cv) {guard} cv = RB_SCH_CV schCheck(cv))"#)
    }

    pub fn open_cellview(&self, lib: &str, cell: &str, view: &str) -> String {
        let lib = escape_skill_string(lib);
        let cell = escape_skill_string(cell);
        let view = escape_skill_string(view);
        // dbOpenCellViewByType with viewType="schematic" mode="a":
        //   creates cellview if absent, opens for editing (non-interactive)
        // Store in RB_SCH_CV global for use by subsequent commands
        format!(r#"RB_SCH_CV = dbOpenCellViewByType("{lib}" "{cell}" "{view}" "schematic" "a")"#)
    }

    pub fn save(&self) -> String {
        let guard = cv_guard();
        format!(r#"let((cv) {guard} cv = RB_SCH_CV dbSave(cv))"#)
    }

    pub fn set_instance_param(&self, inst_name: &str, param: &str, value: &str) -> String {
        let inst_name = escape_skill_string(inst_name);
        let param = escape_skill_string(param);
        let value = escape_skill_string(value);
        let guard = cv_guard();
        format!(
            r#"let((cv inst) {guard} cv = RB_SCH_CV inst = car(setof(i cv~>instances i~>name == "{inst_name}")) when(inst dbReplaceProp(inst "{param}" "string" "{value}")))"#
        )
    }

    // ── Read operations ──────────────────────────────────────────────

    /// List all instances in the open cellview. Returns JSON array via sprintf.
    pub fn list_instances(&self) -> String {
        let guard = cv_guard();
        format!(
            r#"let((cv out sep lib cell) {guard} cv = RB_SCH_CV out = "[" sep = "" foreach(inst cv~>instances lib = if(inst~>master inst~>master~>libName "?") cell = if(inst~>master inst~>master~>cellName "?") out = strcat(out sep sprintf(nil "{{\"name\":\"%s\",\"master\":\"%s/%s\",\"x\":%g,\"y\":%g}}" inst~>name lib cell car(inst~>xy) cadr(inst~>xy))) sep = ",") strcat(out "]"))"#
        )
    }

    /// List all nets in the open cellview. Returns JSON array.
    pub fn list_nets(&self) -> String {
        let guard = cv_guard();
        format!(
            r#"let((cv out sep) {guard} cv = RB_SCH_CV out = "[" sep = "" foreach(net cv~>nets out = strcat(out sep sprintf(nil "\"%s\"" net~>name)) sep = ",") strcat(out "]"))"#
        )
    }

    /// List all pins (terminals) in the open cellview. Returns JSON array.
    pub fn list_pins(&self) -> String {
        let guard = cv_guard();
        format!(
            r#"let((cv out sep) {guard} cv = RB_SCH_CV out = "[" sep = "" foreach(term cv~>terminals out = strcat(out sep sprintf(nil "{{\"name\":\"%s\",\"direction\":\"%s\"}}" term~>name term~>direction)) sep = ",") strcat(out "]"))"#
        )
    }

    /// Get parameters of a specific instance. Returns JSON object.
    pub fn get_instance_params(&self, inst_name: &str) -> String {
        let inst_name = escape_skill_string(inst_name);
        let guard = cv_guard();
        format!(
            r#"let((cv inst out sep v) {guard} cv = RB_SCH_CV inst = car(setof(i cv~>instances strcmp(i~>name "{inst_name}")==0)) if(inst then out = "{{" sep = "" foreach(prop inst~>prop when(prop~>name != nil v = prop~>value when(v out = strcat(out sep sprintf(nil "\"%s\":\"%s\"" prop~>name if(stringp(v) v sprintf(nil "%L" v)))) sep = ","))) strcat(out "}}") else "null"))"#
        )
    }

    /// Assign net name to instance terminal.
    /// Finds the master terminal and creates an instTerm on the named net.
    /// No wire drawing coordinates needed — purely a logical connection.
    pub fn assign_net(&self, inst_name: &str, term_name: &str, net_name: &str) -> String {
        let inst_name = escape_skill_string(inst_name);
        let term_name = escape_skill_string(term_name);
        let net_name = escape_skill_string(net_name);
        format!(
            r#"let((cv inst iterm masterTerm net) cv = RB_SCH_CV inst = car(setof(i cv~>instances strcmp(i~>name "{inst_name}")==0)) masterTerm = car(setof(x inst~>master~>terminals upperCase(x~>name)==upperCase("{term_name}"))) net = dbMakeNet(cv "{net_name}") iterm = car(setof(x inst~>instTerms upperCase(x~>name)==upperCase("{term_name}"))) when(iterm dbDeleteObject(iterm)) when(masterTerm dbCreateInstTerm(net inst masterTerm)))"#
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ops() -> SchematicOps {
        SchematicOps::new()
    }

    #[test]
    fn create_instance_uses_orient() {
        let s = ops().create_instance("analogLib", "nmos4", "symbol", "M1", (100, 200), "MY");
        assert!(s.contains("\"MY\""), "orient must be in SKILL: {s}");
        assert!(
            s.contains("100") && s.contains("200"),
            "origin must be in SKILL: {s}"
        );
        assert!(s.contains("\"M1\""), "instance name must be quoted: {s}");
    }

    #[test]
    fn create_instance_default_orient() {
        let s = ops().create_instance("lib", "cell", "symbol", "X0", (0, 0), "R0");
        assert!(s.contains("\"R0\""), "{s}");
    }

    #[test]
    fn assign_net_uses_db_create_inst_term() {
        let s = ops().assign_net("M1", "G", "VIN");
        assert!(s.contains("dbCreateInstTerm"), "must create an instTerm on the net: {s}");
        assert!(
            !s.contains("schCreateWire"),
            "must not use schCreateWire: {s}"
        );
        assert!(
            !s.contains("0 0"),
            "hardcoded coordinates must be gone: {s}"
        );
    }

    #[test]
    fn assign_net_escapes_names() {
        let s = ops().assign_net(r#"M"1"#, "D", "VDD");
        assert!(s.contains(r#"M\"1"#), "inst name must be escaped: {s}");
    }

    #[test]
    fn open_cellview_sets_global() {
        let s = ops().open_cellview("myLib", "myCell", "schematic");
        assert!(s.starts_with("RB_SCH_CV ="), "{s}");
        assert!(s.contains("\"myLib\"") && s.contains("\"myCell\""), "{s}");
    }

    #[test]
    fn cv_guard_is_injected_in_write_ops() {
        let s = ops().create_wire(&[(0, 0), (10, 10)], "wire", "VDD");
        assert!(s.contains("boundp('RB_SCH_CV)"), "guard must be present: {s}");
        assert!(s.contains("dbCreateWire"), "{s}");
    }

    #[test]
    fn create_wire_label_contains_guard() {
        let s = ops().create_wire_label("GND", (50, 50));
        assert!(s.contains("boundp('RB_SCH_CV)"), "{s}");
    }

    #[test]
    fn create_pin_respects_pin_type() {
        let s = ops().create_pin("Y", "output", (10, 20));
        assert!(s.contains("\"opin\""), "output pin must use basic/opin: {s}");
        assert!(!s.contains("\"ipin\""), "output pin must not use basic/ipin: {s}");
    }

    #[test]
    fn create_pin_supports_inout() {
        let s = ops().create_pin("VDD", "inout", (10, 20));
        assert!(s.contains("\"iopin\""), "inout pin must use basic/iopin: {s}");
    }

    #[test]
    fn clear_current_cellview_deletes_existing_objects() {
        let s = ops().clear_current_cellview();
        assert!(s.contains("dbDeleteObject"), "{s}");
        assert!(s.contains("cv~>instances"), "{s}");
        assert!(s.contains("cv~>terminals"), "{s}");
    }

    #[test]
    fn save_contains_guard() {
        let s = ops().save();
        assert!(s.contains("boundp('RB_SCH_CV)"), "{s}");
        assert!(s.contains("dbSave"), "{s}");
    }
}
