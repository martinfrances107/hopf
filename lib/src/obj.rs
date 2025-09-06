use std::io::BufWriter;
use std::io::Write;

#[derive(Debug, Default)]
struct Obj{
  // a collection of lines for a mesh.
  meshes: Vec<Vec<Vec<(f64, f64, f64)>>>,
}

impl Obj{
  fn add(&mut self, mesh: Vec<Vec<(f64, f64, f64)>>){
    self.meshes.push(mesh);
  }

  /// Generate an OBJ file from a `PointCloud`.
///
/// # Errors
///   When writing to a buffer fails
pub fn write<W>(&self, out: &mut BufWriter<W>) -> Result<(), std::io::Error>
where
 W: ?Sized + std::io::Write
{

    // in OBJ files the index runs to 1...=N
    let mut index = 1;
    for (i, mesh) in self.meshes.iter().enumerate(){
      writeln!(out, "o mesh_{i}")?;
    for (j, line) in mesh.iter().enumerate() {
      writeln!(out, "g line_{j}")?;
        for (x, y, z) in line {
            writeln!(out, "v {x} {y} {z}")?;
        }
        // out.push_str("g hopf_fibration\n");
        write!(out, "l")?;

        // First point of the loop.
        let index0 = index;
        for _ in line{
          write!(out, " {index}")?;
          index += 1;
        }
        // Close the loop by appending the start of the loop to the end.
        writeln!(out, " {index0}")?;

      }
      }

    Ok(())
}
}

