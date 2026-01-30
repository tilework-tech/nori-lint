pub mod line_count;

pub trait Rule {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn run(&self, input: &str) -> Option<String>;
}
