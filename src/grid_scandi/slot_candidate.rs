#[derive(PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct Conflict {
    restricted_slot_id: u32,
    restricting_slot_id: u32
}

impl Conflict {
    pub fn get_restricted_slot_id(&self) -> u32 {
        self.restricted_slot_id
    }

    pub fn get_restricting_slot_id(&self) -> u32 {
        self.restricting_slot_id
    }
}

#[derive(PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct SlotCandidate {
    word: String,
    assigned_slot_id: Option<u32>,
    conflicts: Vec<Conflict>
}

impl SlotCandidate {

    pub fn new(word: String) -> Self 
    {
        Self { word, assigned_slot_id: None, conflicts: Vec::new() }
    }
    
    pub fn get_word(&self) -> &String 
    {
        &self.word
    }

    pub fn get_conflicts(&self) -> &Vec<Conflict> 
    {
        &self.conflicts
    }

    pub fn get_assigned_slot_id(&self) -> Option<u32> 
    {
        self.assigned_slot_id
    }

    pub fn record_conflict(&mut self, restricted_slot_id: u32, restricting_slot_id: u32) -> () 
    {
        self.conflicts.push( Conflict { restricted_slot_id, restricting_slot_id } );
    }

    pub fn set_assigned_slot_id(&mut self, slot_id: u32) -> () 
    {
        self.assigned_slot_id = Some(slot_id);
    }

    pub fn clear_assigned_slot_id(&mut self) -> ()
    {
        self.assigned_slot_id = None;
    }

    pub fn remove_restrictions(&mut self, restricted_slot_id: u32, restricting_slot_id: u32) 
    {
        self.conflicts.retain(|conflict| conflict.get_restricted_slot_id() != restricted_slot_id && conflict.get_restricting_slot_id() != restricting_slot_id);
    }
}