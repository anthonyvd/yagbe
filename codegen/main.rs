use std::path::Path;
use std::fs;
use json;

fn main() {
  let p = Path::new("./Opcodes.json");
  println!("Generating code from {}", p.to_str().unwrap());

  let contents: String = fs::read_to_string(&p).unwrap();
  let parsed = json::parse(&contents).unwrap();

  println!("{} unprefixed instructions and {} prefixed instructions", parsed["unprefixed"].len(), parsed["cbprefixed"].len());

  let mut code = "
use memory_utils::Source;
use memory_utils::Dest;

fn execute_unprefixed(instr: u8) {
  match instr {
".to_string();

  for (k, v) in parsed["unprefixed"].entries() {
    code.push_str(&format!("    {} => self.{}(),\n", k, v["mnemonic"].to_string().to_ascii_lowercase()));
  }

  code.push_str("
  }
}");
  println!("code: \n{}", code);

}