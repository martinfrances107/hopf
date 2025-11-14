static SVG_HEADER: &str = r#"<?xml version="1.0" encoding="UTF-8" standalone="no"?>
<svg
   width="1200px"
   height="120px"
   viewBox="-3 0 357 36"
   version="1.1"
   xmlns="http://www.w3.org/2000/svg"
   xmlns:svg="http://www.w3.org/2000/svg">

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
    fill="black" stroke="black"
    x="{angle}"
    y="29"
    >{angle}</text>"#
            );

            println!(
                r#"<rect
    width="1.0"
    height="20.0"
    fill="white" stroke="white"
    x="{angle}"
    y="0" />"#
            );
        } else if angle % 30 == 15 {
            // Minor tick
            println!(
                r#"<rect
    width="1.0"
    height="10.0"
    fill="white" stroke="white"
    x="{angle}"
    y="0" />"#
            );
        } else {
            // sub tick every 5 degrees
            println!(
                r#"<rect
     width="1.0"
     height="5.0"
     fill="white" stroke="white"
     x="{angle}"
     y="0" />"#
            );
        }
    }
    println!("{}", SVG_FOOTER);
}
