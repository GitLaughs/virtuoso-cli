; RB_connectTerminal - Place wire stub + net label at an instance terminal
; Used by vcli schematic build for non-interactive schematic connections
;
; Parameters:
;   cv       - cellView object (must be open for editing)
;   instName - instance name (e.g. "M5")
;   termName - terminal name (e.g. "D", "G", "S", "B")
;   netName  - net label name (e.g. "vtail", "vdd!")
;
; Creates a short wire path at the terminal pin position and labels it
; with the net name. Terminals sharing the same label are connected.

procedure(RB_connectTerminal(cv instName termName netName)
  let((inst term iterm pinFig pinBox center labelPoint stubEnd termUpper isPmos origin dirPt localDir wireObjs wireObj net)
    inst = car(setof(i cv~>instances strcmp(i~>name instName)==0))
    unless(inst
      printf("  [ERROR] Instance %s not found\n" instName)
      return(nil))
    term = car(setof(x inst~>master~>terminals upperCase(x~>name) == upperCase(termName)))
    unless(term
      printf("  [ERROR] Terminal %s not found on %s\n" termName instName)
      return(nil))
    pinFig = car(car(term~>pins)~>figs)
    unless(pinFig
      printf("  [ERROR] Terminal %s on %s has no pin figure\n" termName instName)
      return(nil))
    pinBox = dbTransformBBox(pinFig~>bBox inst~>transform)
    center = list(
      quotient(plus(xCoord(lowerLeft(pinBox)) xCoord(upperRight(pinBox))) 2.0)
      quotient(plus(yCoord(lowerLeft(pinBox)) yCoord(upperRight(pinBox))) 2.0)
    )
    termUpper = upperCase(termName)
    isPmos = rexMatchp("pmos" lowerCase(inst~>master~>cellName))
    origin = dbTransformPoint(list(0 0) inst~>transform)
    localDir = cond(
      (termUpper == "G" list(-0.45 0))
      (termUpper == "D" list(0.45 0))
      (termUpper == "S" list(-0.45 0))
      (termUpper == "B" if(isPmos list(0.0 0.45) list(0.0 -0.45)))
      (t list(0.45 0))
    )
    dirPt = dbTransformPoint(localDir inst~>transform)
    stubEnd = list(xCoord(center) + (xCoord(dirPt) - xCoord(origin))
                   yCoord(center) + (yCoord(dirPt) - yCoord(origin)))
    labelPoint = list((xCoord(center) + xCoord(stubEnd)) / 2.0
                      (yCoord(center) + yCoord(stubEnd)) / 2.0)
    wireObjs = schCreateWire(cv "route" "full" list(center stubEnd) 0 0 0 nil nil)
    wireObj = car(wireObjs)
    schCreateWireLabel(cv wireObj labelPoint netName "lowerCenter" "R0" "stick" 0.0625 nil)
    net = dbMakeNet(cv netName)
    iterm = car(setof(x inst~>instTerms upperCase(x~>name) == termUpper))
    when(iterm dbDeleteObject(iterm))
    dbCreateInstTerm(net inst term)
    t
  )
)
