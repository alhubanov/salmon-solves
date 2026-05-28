pub trait Grid {
    fn initialize(width: u32, height: u32) -> Self;
    fn construct(&mut self) -> ();
    fn print(&self) -> ();
}