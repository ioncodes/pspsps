fn main() {
    println!("cargo:rerun-if-changed=src/grammar/instruction.pest");
}