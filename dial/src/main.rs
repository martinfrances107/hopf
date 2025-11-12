static SVG_HEADER: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<svg
   width="360mm"
   height="100mm"
   viewBox="-20 -20 380 70"
   version="1.1"
   xmlns="http://www.w3.org/2000/svg"
   xmlns:svg="http://www.w3.org/2000/svg">

   <rect x="-20" y="-20" width="400" height="90" style="fill:#7F7F7FFF" />
    <g
      text-anchor="middle"
      style="font-weight:900;font-size:9.525px;font-family:Timmana;-inkscape-font-specification:'Timmana, Heavy';stroke-width:0.0132292;stroke-linecap:round;stroke-linejoin:round"
    >
"#;

static SVG_FOOTER: &str = r#"</g>

</svg>"#;

fn main() {
    println!("{}", SVG_HEADER);

    for angle in (0..360).step_by(5) {
        if angle % 30 == 0 {
            // TODO must add text centering
            // Major tick has text
            println!(
                r#"<text
    style="fill:#000000FF;stroke:#000000FF"
    x="{angle}"
    y="29"
    >{angle}</text>"#
            );

            println!(
                r#"<rect
    width="1.0"
    height="20.0"
    style="fill:#FFFFFFFF;stroke:#FFFFFFFF"
    x="{angle}"
    y="0" />"#
            );
        } else if angle % 30 == 15 {
            // Minor tick
            println!(
                r#"<rect
    width="1.0"
    height="10.0"
    style="fill:#FFFFFFFF;stroke:#FFFFFFFF"
    x="{angle}"
    y="0" />"#
            );
        } else {
            // sub tick every 5 degrees
            println!(
                r#"<rect
     width="1.0"
     height="5.0"
     style="fill:#FFFFFFFF;stroke:#FFFFFFFF"
     x="{angle}"
     y="0" />"#
            );
        }
    }
    println!("{}", SVG_FOOTER);
}
