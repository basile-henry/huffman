use bit_vec::BitVec;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::fmt::Debug;
use std::hash::Hash;
use std::iter::FromIterator;

pub fn encode<S: Eq + Hash + Copy + Ord + Debug>(
    content: &[S],
) -> Option<(HuffmanKey<S>, Vec<u8>)> {
    let decode_key = make_key(content)?;
    let encode_key = make_encode_key(&decode_key);

    // We preallocate memory for the encoded data, it should take as much space
    // as the `content` in the worse case
    let mut encoded_bits = BitVec::with_capacity(std::mem::size_of::<S>() * content.len());

    for ref symbol in content {
        match encode_key.get(&Some(**symbol)) {
            None => panic!("Unexpected: Symbol not in the encode_key!"),
            Some(bits) => encoded_bits.extend(bits),
        }
    }

    // Add the end of input
    match encode_key.get(&None) {
        None => panic!("Unexpected: End of input not in the encode_key!"),
        Some(bits) => encoded_bits.extend(bits),
    }

    encoded_bits.shrink_to_fit();

    Some((decode_key, encoded_bits.to_bytes()))
}

pub fn decode<S: Copy + Debug>(key: &HuffmanKey<S>, encoded: &[u8]) -> Option<Vec<S>> {
    let mut bits = BitVec::from_bytes(encoded).into_iter();
    let mut out = Vec::new();

    loop {
        match decode_symbol(key, &mut bits) {
            DecodedSymbol::NotEnoughBits => return None,
            DecodedSymbol::EndOfInput => return Some(out),
            DecodedSymbol::Symbol(s) => out.push(s),
        }
    }
}

enum DecodedSymbol<S> {
    NotEnoughBits,
    EndOfInput,
    Symbol(S),
}

fn decode_symbol<S: Copy + Debug>(
    key: &HuffmanKey<S>,
    bits: &mut Iterator<Item = bool>,
) -> DecodedSymbol<(S)> {
    let mut tree = key;

    loop {
        match tree {
            HuffmanKey::EndOfInput => return DecodedSymbol::EndOfInput,
            HuffmanKey::Symbol(s) => return DecodedSymbol::Symbol(*s),
            HuffmanKey::Branch { left, right } => match bits.next() {
                None => return DecodedSymbol::NotEnoughBits,
                Some(true) => tree = right,
                Some(false) => tree = left,
            },
        }
    }
}

#[derive(Debug)]
pub enum HuffmanKey<S> {
    EndOfInput,
    Symbol(S),
    Branch {
        left: Box<HuffmanKey<S>>,
        right: Box<HuffmanKey<S>>,
    },
}

#[derive(Eq, PartialEq, Debug)]
pub enum HuffmanTree<S> {
    Leaf {
        frequency: u64,
        symbol: S,
    },
    Branch {
        frequency: u64,
        left: Box<HuffmanTree<S>>,
        right: Box<HuffmanTree<S>>,
    },
}

impl<S: Eq + Ord> Ord for HuffmanTree<S> {
    fn cmp(&self, other: &Self) -> Ordering {
        // The comparison of nodes is the inverse of the frequency in order to
        // work well with the max-heap that is BinaryHeap
        match Ord::cmp(&frequency(&self), &frequency(&other)) {
            Ordering::Equal => Ordering::Equal,
            Ordering::Greater => Ordering::Less,
            Ordering::Less => Ordering::Greater,
        }
    }
}

impl<S: Eq + Ord> PartialOrd for HuffmanTree<S> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn symbol_frequency<S: Eq + Hash + Copy>(content: &[S]) -> HashMap<S, u64> {
    let mut symbol_counts = HashMap::new();

    for symbol in content {
        *symbol_counts.entry(*symbol).or_insert(0) += 1;
    }

    symbol_counts
}

fn frequency<S>(tree: &HuffmanTree<S>) -> u64 {
    match tree {
        HuffmanTree::Leaf { frequency, .. } | HuffmanTree::Branch { frequency, .. } => *frequency,
    }
}

fn make_key<S: Eq + Hash + Copy + Ord + Debug>(content: &[S]) -> Option<HuffmanKey<S>> {
    let frequency_map = symbol_frequency(&content);

    let mut priority_queue =
        BinaryHeap::from_iter(frequency_map.iter().map(|(&s, &v)| HuffmanTree::Leaf {
            frequency: v,
            symbol: Some(s),
        }));

    // Push the end of input symbol with lowest frequency
    priority_queue.push(HuffmanTree::Leaf {
        frequency: 0,
        symbol: None,
    });

    loop {
        if let Some(a) = priority_queue.pop() {
            if let Some(b) = priority_queue.pop() {
                let a_freq = frequency(&a);
                let b_freq = frequency(&b);
                let branch = if a_freq < b_freq {
                    HuffmanTree::Branch {
                        frequency: a_freq + b_freq,
                        left: Box::new(a),
                        right: Box::new(b),
                    }
                } else {
                    HuffmanTree::Branch {
                        frequency: a_freq + b_freq,
                        left: Box::new(b),
                        right: Box::new(a),
                    }
                };

                priority_queue.push(branch);
            } else {
                return Some(tree_to_key(a));
            }
        } else {
            panic!("Empty priority queue");
        }
    }
}

fn tree_to_key<S>(tree: HuffmanTree<Option<S>>) -> HuffmanKey<S> {
    match tree {
        HuffmanTree::Leaf { symbol: None, .. } => HuffmanKey::EndOfInput,
        HuffmanTree::Leaf {
            symbol: Some(s), ..
        } => HuffmanKey::Symbol(s),
        HuffmanTree::Branch { left, right, .. } => HuffmanKey::Branch {
            left: Box::new(tree_to_key(*left)),
            right: Box::new(tree_to_key(*right)),
        },
    }
}

fn make_encode_key<S: Eq + Hash + Copy + Debug>(key: &HuffmanKey<S>) -> HashMap<Option<S>, BitVec> {
    let mut map = HashMap::new();
    let mut stack = Vec::new();
    stack.push((BitVec::new(), key));

    while let Some((bits, key)) = stack.pop() {
        match key {
            HuffmanKey::EndOfInput => {
                map.insert(None, bits);
            }
            HuffmanKey::Symbol(symbol) => {
                map.insert(Some(*symbol), bits);
            }
            HuffmanKey::Branch { left, right } => {
                let mut key_left = bits.clone();
                key_left.push(false);
                stack.push((key_left, left));

                let mut key_right = bits.clone();
                key_right.push(true);
                stack.push((key_right, right));
            }
        }
    }
    map
}
