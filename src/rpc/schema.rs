//! RPC schema — method signatures and parameter metadata.
//!
//! Each method has:
//!   - `name`: "domain.method" namespaced by domain
//!   - `params`: JSON Schema-like parameter list
//!   - `returns`: return type description

use serde::{Deserialize, Serialize};

/// A single RPC method definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Method {
    pub name: String,
    pub summary: String,
    pub params: Vec<Param>,
    pub returns: String,
}

/// A parameter definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub ptype: String,
    pub description: String,
    pub required: bool,
}

/// Full RPC schema containing all available methods.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcSchema {
    pub version: String,
    pub methods: Vec<Method>,
}

impl RpcSchema {
    pub fn new(methods: Vec<Method>) -> Self {
        Self {
            version: "1.0".into(),
            methods,
        }
    }
}

/// Built-in schema with all available RPC methods.
pub fn standard_schema() -> RpcSchema {
    RpcSchema::new(vec![
        // ── Schematic ────────────────────────────────────────────────
        Method {
            name: "schematic.open_cell_view".into(),
            summary: "Open or create a schematic cellview for editing".into(),
            params: vec![
                Param {
                    name: "lib".into(),
                    ptype: "string".into(),
                    description: "Library name".into(),
                    required: true,
                },
                Param {
                    name: "cell".into(),
                    ptype: "string".into(),
                    description: "Cell name".into(),
                    required: true,
                },
                Param {
                    name: "view".into(),
                    ptype: "string".into(),
                    description: "View name (default: schematic)".into(),
                    required: false,
                },
            ],
            returns: "null on success".into(),
        },
        Method {
            name: "schematic.place".into(),
            summary: "Place an instance in the open schematic".into(),
            params: vec![
                Param {
                    name: "master".into(),
                    ptype: "string".into(),
                    description: "Master cell in lib/cell format (e.g. smic13mmrf/p12)".into(),
                    required: true,
                },
                Param {
                    name: "name".into(),
                    ptype: "string".into(),
                    description: "Instance name".into(),
                    required: true,
                },
                Param {
                    name: "x".into(),
                    ptype: "integer".into(),
                    description: "X coordinate".into(),
                    required: false,
                },
                Param {
                    name: "y".into(),
                    ptype: "integer".into(),
                    description: "Y coordinate".into(),
                    required: false,
                },
                Param {
                    name: "orient".into(),
                    ptype: "string".into(),
                    description: "Orientation (R0, R90, R180, R270, MY, MX, etc.)".into(),
                    required: false,
                },
            ],
            returns: "null on success".into(),
        },
        Method {
            name: "schematic.wire".into(),
            summary: "Create a wire between named net and coordinates".into(),
            params: vec![
                Param {
                    name: "net".into(),
                    ptype: "string".into(),
                    description: "Net name".into(),
                    required: true,
                },
                Param {
                    name: "points".into(),
                    ptype: "array".into(),
                    description: "Points as x1,y1 x2,y2 ...".into(),
                    required: true,
                },
            ],
            returns: "null on success".into(),
        },
        Method {
            name: "schematic.label".into(),
            summary: "Add a net label at coordinates".into(),
            params: vec![
                Param {
                    name: "net".into(),
                    ptype: "string".into(),
                    description: "Net name".into(),
                    required: true,
                },
                Param {
                    name: "x".into(),
                    ptype: "integer".into(),
                    description: "X coordinate".into(),
                    required: false,
                },
                Param {
                    name: "y".into(),
                    ptype: "integer".into(),
                    description: "Y coordinate".into(),
                    required: false,
                },
            ],
            returns: "null on success".into(),
        },
        Method {
            name: "schematic.pin".into(),
            summary: "Add a pin to a net".into(),
            params: vec![
                Param {
                    name: "net".into(),
                    ptype: "string".into(),
                    description: "Net name".into(),
                    required: true,
                },
                Param {
                    name: "direction".into(),
                    ptype: "string".into(),
                    description: "Pin direction: input, output, inputOutput".into(),
                    required: true,
                },
                Param {
                    name: "x".into(),
                    ptype: "integer".into(),
                    description: "X coordinate".into(),
                    required: false,
                },
                Param {
                    name: "y".into(),
                    ptype: "integer".into(),
                    description: "Y coordinate".into(),
                    required: false,
                },
            ],
            returns: "null on success".into(),
        },
        Method {
            name: "schematic.save".into(),
            summary: "Save the current schematic".into(),
            params: vec![],
            returns: "null on success".into(),
        },
        Method {
            name: "schematic.check".into(),
            summary: "Run schematic check (schCheck)".into(),
            params: vec![],
            returns: "schCheck output".into(),
        },
        Method {
            name: "schematic.list_instances".into(),
            summary: "List all instances in the open cellview".into(),
            params: vec![],
            returns: "JSON array of instances".into(),
        },
        Method {
            name: "schematic.list_nets".into(),
            summary: "List all nets in the open cellview".into(),
            params: vec![],
            returns: "JSON array of net names".into(),
        },
        Method {
            name: "schematic.list_pins".into(),
            summary: "List all pins in the open cellview".into(),
            params: vec![],
            returns: "JSON array of pins".into(),
        },
        Method {
            name: "schematic.get_params".into(),
            summary: "Get parameters of a specific instance".into(),
            params: vec![Param {
                name: "inst".into(),
                ptype: "string".into(),
                description: "Instance name (e.g. M1)".into(),
                required: true,
            }],
            returns: "JSON object of param name→value".into(),
        },
        // ── Window ────────────────────────────────────────────────────
        Method {
            name: "window.list".into(),
            summary: "List all open Virtuoso windows".into(),
            params: vec![],
            returns: "JSON array of window names".into(),
        },
        Method {
            name: "window.screenshot".into(),
            summary: "Capture screenshot of current window".into(),
            params: vec![Param {
                name: "path".into(),
                ptype: "string".into(),
                description: "Output PNG file path".into(),
                required: true,
            }],
            returns: "file path on success".into(),
        },
        // ── Cell ─────────────────────────────────────────────────────
        Method {
            name: "cell.open".into(),
            summary: "Open a cellview".into(),
            params: vec![
                Param {
                    name: "lib".into(),
                    ptype: "string".into(),
                    description: "Library name".into(),
                    required: true,
                },
                Param {
                    name: "cell".into(),
                    ptype: "string".into(),
                    description: "Cell name".into(),
                    required: true,
                },
                Param {
                    name: "view".into(),
                    ptype: "string".into(),
                    description: "View name".into(),
                    required: false,
                },
                Param {
                    name: "mode".into(),
                    ptype: "string".into(),
                    description: "Open mode: r(ead), o(verwrite), a(ppend)".into(),
                    required: false,
                },
            ],
            returns: "null on success".into(),
        },
        Method {
            name: "cell.save".into(),
            summary: "Save the current cellview".into(),
            params: vec![],
            returns: "null on success".into(),
        },
        Method {
            name: "cell.close".into(),
            summary: "Close the current cellview".into(),
            params: vec![],
            returns: "null on success".into(),
        },
        // ── Maestro ───────────────────────────────────────────────────
        Method {
            name: "maestro.open_session".into(),
            summary: "Open a Maestro session".into(),
            params: vec![
                Param {
                    name: "lib".into(),
                    ptype: "string".into(),
                    description: "Library name".into(),
                    required: true,
                },
                Param {
                    name: "cell".into(),
                    ptype: "string".into(),
                    description: "Cell name".into(),
                    required: true,
                },
                Param {
                    name: "view".into(),
                    ptype: "string".into(),
                    description: "View name".into(),
                    required: false,
                },
            ],
            returns: "session handle string".into(),
        },
        Method {
            name: "maestro.close_session".into(),
            summary: "Close a Maestro session".into(),
            params: vec![Param {
                name: "session".into(),
                ptype: "string".into(),
                description: "Session ID (e.g. fnxSession4)".into(),
                required: true,
            }],
            returns: "null on success".into(),
        },
        Method {
            name: "maestro.list_sessions".into(),
            summary: "List all active Maestro sessions".into(),
            params: vec![],
            returns: "JSON array of session objects".into(),
        },
        Method {
            name: "maestro.set_var".into(),
            summary: "Set a design variable".into(),
            params: vec![
                Param {
                    name: "name".into(),
                    ptype: "string".into(),
                    description: "Variable name".into(),
                    required: true,
                },
                Param {
                    name: "value".into(),
                    ptype: "string".into(),
                    description: "Variable value".into(),
                    required: true,
                },
            ],
            returns: "null on success".into(),
        },
        Method {
            name: "maestro.get_var".into(),
            summary: "Get a design variable".into(),
            params: vec![Param {
                name: "name".into(),
                ptype: "string".into(),
                description: "Variable name".into(),
                required: true,
            }],
            returns: "variable value string".into(),
        },
        Method {
            name: "maestro.list_vars".into(),
            summary: "List all design variables".into(),
            params: vec![],
            returns: "JSON array of {name, value}".into(),
        },
        Method {
            name: "maestro.run".into(),
            summary: "Run simulation asynchronously".into(),
            params: vec![Param {
                name: "session".into(),
                ptype: "string".into(),
                description: "Session ID".into(),
                required: true,
            }],
            returns: "null on success".into(),
        },
        Method {
            name: "maestro.save".into(),
            summary: "Save Maestro setup to disk".into(),
            params: vec![Param {
                name: "session".into(),
                ptype: "string".into(),
                description: "Session ID".into(),
                required: true,
            }],
            returns: "null on success".into(),
        },
        Method {
            name: "maestro.export".into(),
            summary: "Export results to CSV".into(),
            params: vec![
                Param {
                    name: "session".into(),
                    ptype: "string".into(),
                    description: "Session ID".into(),
                    required: true,
                },
                Param {
                    name: "path".into(),
                    ptype: "string".into(),
                    description: "Output CSV file path".into(),
                    required: true,
                },
                Param {
                    name: "test_name".into(),
                    ptype: "string".into(),
                    description: "Test name (optional)".into(),
                    required: false,
                },
            ],
            returns: "null on success".into(),
        },
    ])
}
