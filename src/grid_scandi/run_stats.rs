pub struct RunStats 
{
    total_backtracks: u32,
    total_restarts: u32,
    backtracks_per_attempt: Vec<u32>,
}

impl RunStats 
{
    pub fn new() -> Self 
    {
        RunStats { total_backtracks: 0, total_restarts: 0, backtracks_per_attempt: Vec::new() }
    }

    pub fn get_total_bactracks(&self) -> u32
    {
        self.total_backtracks
    }

    pub fn get_total_restarts(&self) -> u32 
    {
        self.total_restarts
    }

    pub fn get_backtracks_per_attempt(&self) -> &Vec<u32>
    {
        &self.backtracks_per_attempt
    }

    pub fn increment_total_backtracks(&mut self) -> ()
    {
        self.total_backtracks += 1;
    }

    pub fn increment_total_restarts(&mut self) -> ()
    {
        self.total_restarts += 1;
    }

    pub fn record_backtracking_count_for_attempt(&mut self, backtracking_count: u32) -> ()
    {
        self.backtracks_per_attempt.push(backtracking_count);
    }

    pub fn print(&self) -> ()
    {
        println!();
        println!("Backtracked {} times in total, {} restarts.", self.total_backtracks, self.total_restarts);
        
        for (idx, bactracks_per_attempt) in self.backtracks_per_attempt.iter().enumerate()
        {
            println!("  For attempt {}, there were {} backtrackings.", idx + 1, bactracks_per_attempt)
        }
    }
}