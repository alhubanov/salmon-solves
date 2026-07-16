use fixedbitset::FixedBitSet;
use ahash::AHashMap;
use super::slot_candidate::SlotCandidate;

const ALPHABET_SIZE: usize = 26; 

fn char_to_idx(c: char) -> Option<usize> 
{
    match c 
    {
        'a'..='z' => Some(c as usize - 'a' as usize),
        _ => None,
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct LengthCollection 
{
    words: Vec<SlotCandidate>,
    bitsets_per_position: Vec<Vec<FixedBitSet>>
}

impl LengthCollection 
{
    fn get_words_mut(&mut self) -> &mut Vec<SlotCandidate>
    {
        &mut self.words
    }

    fn build(words: Vec<SlotCandidate>, length: usize) -> Self 
    {
        let n = words.len();
        let mut bitsets_per_position: Vec<Vec<FixedBitSet>> = (0..length).map(|_| (0..ALPHABET_SIZE).map(|_| FixedBitSet::with_capacity(n)).collect()).collect();

        for (id, candidate) in words.iter().enumerate() 
        {
            for (pos, letter) in candidate.get_word().chars().enumerate() 
            {
                if let Some(letter_idx) = char_to_idx(letter) 
                {
                    // This sets bit at idx id in the bitset to 1
                    bitsets_per_position[pos][letter_idx].insert(id);
                }
            }
        }

        LengthCollection { words, bitsets_per_position }
    }

    fn query(&self, pattern: &[Option<char>]) -> FixedBitSet 
    {
        let n = self.words.len();
        let mut candidates = FixedBitSet::with_capacity(n);
        
        // Set every bit to 1.
        candidates.insert_range(..);

        for (pos, cell) in pattern.iter().enumerate() 
        {
            if let Some(letter) = cell 
            {
                if let Some(letter_idx) = char_to_idx(*letter) 
                {
                    candidates.intersect_with(&self.bitsets_per_position[pos][letter_idx]);
                } 
                else 
                {
                    panic!("Unknown letter detected.");
                }
            }
        }
        candidates
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Dictionary 
{
    words_per_length: AHashMap<usize, LengthCollection>
}

impl Dictionary
{
    pub fn build(words_file_content: &str) -> Self 
    {
        let mut words_per_length: AHashMap<usize, Vec<SlotCandidate>> = AHashMap::new();

        for line in words_file_content.lines() 
        {
            let word = line.trim();
            if word.is_empty() 
            { 
                continue; 
            }
            words_per_length.entry(word.len()).or_default().push(SlotCandidate::new(word.to_string()));
        }

        let words_per_length = words_per_length
                                .into_iter()
                                .map(|(length, words)| (length, LengthCollection::build(words, length)))
                                .collect();

        Dictionary { words_per_length }
    }

    pub fn get_candidates(&self, length: usize, pattern: &Vec<Option<char>>) -> Vec<&SlotCandidate>
    {
        let letter_idx = self.words_per_length.get(&length).unwrap();
        let bitset = letter_idx.query(pattern);

        bitset.ones().map(|id| &letter_idx.words[id]).collect()
    }

    pub fn get_words_for_length_mut(&mut self, length: usize) -> &mut Vec<SlotCandidate>
    {
        self.words_per_length.get_mut(&length).unwrap().get_words_mut()
    }

    pub fn get_words(&self) -> &AHashMap<usize, LengthCollection>
    {
        &self.words_per_length
    }
}

