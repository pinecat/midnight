/// Trait to implement chomp (removes newline from end of a [String], if there is one)
///
/// Inspired by Ruby's chomp. If it's edible, you can chomp it!
pub trait Edible {
    fn chomp(&mut self);
}

/// Simple implementation for chomp on a [String]
impl Edible for String {
    fn chomp(&mut self) {
        if let Some('\n') = self.chars().next_back() {
            self.pop();
        }
    }
}
