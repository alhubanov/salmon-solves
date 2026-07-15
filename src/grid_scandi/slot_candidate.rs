#[derive(PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct SlotCandidate {
    word: String,
    assigned_slot_id: Option<u32>
}

impl SlotCandidate {

    pub fn new(word: String) -> Self 
    {
        Self { word, assigned_slot_id: None }
    }
    
    pub fn get_word(&self) -> &String 
    {
        &self.word
    }

    pub fn get_assigned_slot_id(&self) -> Option<u32> 
    {
        self.assigned_slot_id
    }

    pub fn set_assigned_slot_id(&mut self, slot_id: u32) -> () 
    {
        self.assigned_slot_id = Some(slot_id);
    }

    pub fn clear_assigned_slot_id(&mut self) -> ()
    {
        self.assigned_slot_id = None;
    }
}