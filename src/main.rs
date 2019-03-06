use serde::Deserialize;
use std::error::Error;
use std::iter::FromIterator;
use permutator::Combination;
use std::collections::BTreeSet;

fn mushes() -> Result<Vec<Mush>, Box<Error>> {
    let f = std::fs::File::open("assets/agaricus-lepiota.data")?;
    let mut rdr = csv::ReaderBuilder::new().has_headers(false).from_reader(f);

    let mut result = Vec::new();
    for r in rdr.deserialize() {
        result.push(r?);
    }
    Ok(result)
}

fn main() {
    let mush = mushes().expect("error getting mushy");
    println!("got {} rows", mush.len());

    // let mut nodes = Vec::new();

    // Nodes to check.  Vec of (rows, node_idx)
    let mut half_nodes: Vec<(_, Option<usize>)> = Vec::new();
    half_nodes.push((mush, None));

    while let Some((mush, node_idx)) = half_nodes.pop() {
        let questions = Questions::new(&mush);
        let best_q = questions
            .map(|q| q.answer(&mush))
            .max_by(|m1, m2| m1.impurity().partial_cmp(&m2.impurity()).unwrap());
        dbg!(best_q);
    }
}

struct Questions<'a, I> where I : Iterator {
    mush: &'a [Mush],
    facet: usize,
}

impl<'a> Questions<'a> {
    fn new(mush: &'a [Mush]) -> Questions {
        Questions { 
            mush, 
            facet: 0,
        }
    }
}

impl<'a> Iterator for Questions<'a> {
    type Item = Question;
    fn next(&mut self) -> Option<Self::Item> {
        self.
    }
}

#[derive(Clone, Debug)]
struct Question {
    facet: usize,
    vals: BTreeSet<char>,
}

impl Question {
    /// Applies the question to the group, separating it into two.
    fn answer(&self, input: &[Mush]) -> Answer {
        let (yes, no) = input
            .iter()
            .partition(|mush| self.vals.contains(&mush.attrs[self.facet]));
        Answer {
            question: self.clone(),
            yes,
            no,
        }
    }
}

#[test]
fn test_answer() {
    let q = Question {
        facet: 0,
        vals: ['a', 'b', 'c'].iter().cloned().collect(),
    };
    let mushs = [
        Mush {
            poison: 'p',
            attrs: ['a'; 22],
        },
        Mush {
            poison: 'p',
            attrs: ['b'; 22],
        },
        Mush {
            poison: 'p',
            attrs: ['c'; 22],
        },
        Mush {
            poison: 'p',
            attrs: ['d'; 22],
        },
        Mush {
            poison: 'p',
            attrs: ['e'; 22],
        },
    ];
    let a = q.answer(&mushs);
    assert_eq!(a.yes.len(), 3);
    assert_eq!(a.no.len(), 2);
}

#[derive(Debug)]
struct Answer {
    question: Question,
    yes: Vec<Mush>,
    no: Vec<Mush>,
}

impl Answer {
    fn gini_two_class_impurity(yes: &[Mush], no: &[Mush]) -> f64 {
        let gini = |v: &[Mush]| {
            let p = v.iter().filter(|m| m.poison == 'p').count() as f64;
            let l = v.len() as f64;
            2.0 * (p / l) * ((l - p) / l)
        };
        gini(yes) + gini(no)
    }

    fn impurity(&self) -> f64 {
        Answer::gini_two_class_impurity(&self.yes, &self.no)
    }
}

#[test]
fn test_impurity() {
    let poisons: Vec<_> = (0..10)
        .map(|_| Mush {
            poison: 'p',
            attrs: ['a'; 22],
        })
        .collect();
    let edibles: Vec<_> = (0..10)
        .map(|_| Mush {
            poison: 'e',
            attrs: ['a'; 22],
        })
        .collect();
    let mixed: Vec<_> = (0..10)
        .map(|i| Mush {
            poison: if i % 2 == 0 { 'e' } else { 'p' },
            attrs: ['a'; 22],
        })
        .collect();

    assert!(
        Answer::gini_two_class_impurity(&poisons, &edibles)
            < Answer::gini_two_class_impurity(&poisons, &mixed)
    );
    assert!(
        (Answer::gini_two_class_impurity(&poisons, &mixed)
            - Answer::gini_two_class_impurity(&mixed, &poisons)).abs() < std::f64::EPSILON;
    );
    assert!(
        Answer::gini_two_class_impurity(&edibles, &poisons)
            < Answer::gini_two_class_impurity(&poisons, &mixed)
    );
}

struct Node {
    rows: Vec<Mush>,
    answer: Answer,
}

const NUM_FACETS: usize = 22;

#[derive(Deserialize, Debug, Clone, Copy)]
struct Mush {
    poison: char,
    attrs: [char; NUM_FACETS],
}
