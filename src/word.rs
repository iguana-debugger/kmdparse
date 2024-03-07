#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Word {
    /// An instruction, represented as 4 bytes. kmdparse handles flipping the bytes, so that
    /// instructions are the right way around.
    Instruction([u8; 4]),
    Data(Vec<u8>),
}
