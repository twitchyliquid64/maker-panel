//! Generates drill files.

use super::InnerAtom;
use std::collections::HashMap;

pub fn serialize<W: std::io::Write>(
    features: &Vec<InnerAtom>,
    w: &mut W,
    want_plated: bool,
) -> Result<(), std::io::Error> {
    w.write(b"M48\n")?; // Start of header
    w.write(b";DRILL file {KiCad 5.0.2 compatible}\n")?;
    w.write(b";FORMAT={-:-/ absolute / inch / decimal}\n")?;
    w.write(b"FMAT,2\n")?; // Uses format 2 commands
    w.write(b"INCH,TZ\n")?; // Units are inches, trailing zeroes included.

    let mut circle_dia = HashMap::new();
    for f in features {
        if let InnerAtom::Drill {
            center: _,
            radius,
            plated,
        } = f
        {
            if want_plated == *plated {
                let dia_inches = format!("{:.4}", radius * 2.0 / 25.4);
                circle_dia.insert(dia_inches, ());
            }
        }
    }
    let circle_tools: Vec<_> = circle_dia.keys().enumerate().collect();
    for (i, c) in &circle_tools {
        w.write(format!("T{}C{}\n", i + 1, c).as_bytes())?;
    }
    w.write(b"%\n")?; // Rewind, used instead of end of header M95.

    w.write(b"G90\n")?; // Absolute mode
    w.write(b"G05\n")?; // Turn on drill mode

    let mut current_tool: Option<usize> = None;
    for f in features {
        if let InnerAtom::Drill {
            center,
            radius,
            plated,
        } = f
        {
            if want_plated == *plated {
                let dia_inches = format!("{:.4}", radius * 2.0 / 25.4);
                let tool_idx = circle_tools
                    .iter()
                    .find(|&&(_, dia)| *dia == dia_inches)
                    .unwrap()
                    .0;
                if current_tool != Some(tool_idx + 1) {
                    w.write(format!("T{}\n", tool_idx + 1).as_bytes())?;
                    current_tool = Some(tool_idx + 1);
                }

                let (x, y) = (center.x / 25.4, center.y / 25.4);
                w.write(format!("X{:.4}Y{:.4}\n", x, y).as_bytes())?;
            }
        }
    }

    w.write(b"T0\n")?; // Remove tool from spindle.
    w.write(b"M30\n")?; // End of file (last line)
    Ok(())
}

// FMAT,2
// INCH,TZ
// T1C0.1220
// %
// G90
// G05
// T1
// X0.8031Y-0.1673
// X0.Y-0.1673
// X0.8031Y0.1673
// X-0.8031Y-0.1673
// X0.4016Y-0.1673
// X-0.4016Y-0.1673
// X-0.4016Y0.1673
// X0.Y0.1673
// X-0.8031Y0.1673
// X0.4016Y0.1673
// T0
// M30
