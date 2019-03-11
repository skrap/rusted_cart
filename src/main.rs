use lazy_static;
use permutator::Combination;
use serde::Deserialize;
use std::collections::{BTreeSet, HashMap};
use std::error::Error;

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

    // Nodes to check.  Vec of (rows, node_idx)
    let mut half_nodes: Vec<(_, usize)> = Vec::new();
    half_nodes.push((mush, 1));

    let mut next_page = 2;

    while let Some((mush, page)) = half_nodes.pop() {
        let mut min_impurinty = None;
        for facet in 0..NUM_FACETS {
            let vals = &facet_vals(&mush, facet);
            let questions =
                (1..vals.len())
                    .flat_map(move |k| vals.combination(k))
                    .map(move |combis| Question {
                        facet,
                        vals: combis.into_iter().cloned().collect(),
                    });

            for question in questions {
                let answer = question.answer(&mush);
                let ans_imp = answer.impurity;
                if let Some((min_i, _, _)) = min_impurinty {
                    if ans_imp < min_i {
                        min_impurinty = Some((ans_imp, question, answer));
                    }
                } else {
                    min_impurinty = Some((ans_imp, question, answer));
                }
            }
        }
        match min_impurinty {
            Some((_imp, quest, ans)) => {
                println!("page {}: {}", page, quest);
                for (txt, node) in &[("yes", &ans.yes), ("no", &ans.no)] {
                    if node.impurity == 0.0 {
                        println!("\tif {}, done. {}", txt, node);
                    } else {
                        next_page += 1;
                        println!("\tif {}, {}, goto page {}", txt, node, next_page);
                        half_nodes.push((node.rows.clone(), next_page));
                    }
                }
            }
            None => panic!("huh? no nodes or sumpin?"),
        }
    }
}

fn facet_vals(mushs: &[Mush], facet: usize) -> Vec<char> {
    mushs
        .iter()
        .map(|m| m.attrs[facet])
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
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
        Answer::new(yes, no)
    }
}

impl std::fmt::Display for Question {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (facet_name, facet_map) = &FACETS[self.facet];
        let choices = facet_map
            .iter()
            .filter_map(|(k, v)| if self.vals.contains(k) { Some(v) } else { None })
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        write!(
            f,
            "Examine '{}'.  Is it {}{}?",
            facet_name,
            if self.vals.len() > 1 { "one of" } else { "" },
            choices
        )
    }
}

#[test]
fn test_question_fmt() {
    use std::iter::FromIterator;
    let q = Question {
        facet: 0,
        vals: BTreeSet::from_iter(['b', 'c', 'x'].iter().cloned()),
    };
    format!("{}", q);
}

lazy_static::lazy_static! {
    static ref FACETS: Vec<(&'static str,HashMap<char,&'static str>)> = {
        let facet_data = [
                            ("cap-shape"                ,"bell=b,conical=c,convex=x,flat=f,knobbed=k,sunken=s"),
                            ("cap-surface"              ,"fibrous=f,grooves=g,scaly=y,smooth=s"),
                            ("cap-color"                ,"brown=n,buff=b,cinnamon=c,gray=g,green=r,pink=p,purple=u,red=e,white=w,yellow=y"),
                            ("bruises?"                 ,"bruises=t,no=f"),
                            ("odor"                     ,"almond=a,anise=l,creosote=c,fishy=y,foul=f,musty=m,none=n,pungent=p,spicy=s"),
                            ("gill-attachment"          ,"attached=a,descending=d,free=f,notched=n"),
                            ("gill-spacing"             ,"close=c,crowded=w,distant=d"),
                            ("gill-size"                ,"broad=b,narrow=n"),
                            ("gill-color"               ,"black=k,brown=n,buff=b,chocolate=h,gray=g,green=r,orange=o,pink=p,purple=u,red=e,white=w,yellow=y"),
                            ("stalk-shape"              ,"enlarging=e,tapering=t"),
                            ("stalk-root"               ,"bulbous=b,club=c,cup=u,equal=e,rhizomorphs=z,rooted=r,missing=?"),
                            ("stalk-surface-above-ring" ,"fibrous=f,scaly=y,silky=k,smooth=s"),
                            ("stalk-surface-below-ring" ,"fibrous=f,scaly=y,silky=k,smooth=s"),
                            ("stalk-color-above-ring"   ,"brown=n,buff=b,cinnamon=c,gray=g,orange=o,pink=p,red=e,white=w,yellow=y"),
                            ("stalk-color-below-ring"   ,"brown=n,buff=b,cinnamon=c,gray=g,orange=o,pink=p,red=e,white=w,yellow=y"),
                            ("veil-type"                ,"partial=p,universal=u"),
                            ("veil-color"               ,"brown=n,orange=o,white=w,yellow=y"),
                            ("ring-number"              ,"none=n,one=o,two=t"),
                            ("ring-type"                ,"cobwebby=c,evanescent=e,flaring=f,large=l,none=n,pendant=p,sheathing=s,zone=z"),
                            ("spore-print-color"        ,"black=k,brown=n,buff=b,chocolate=h,green=r,orange=o,purple=u,white=w,yellow=y"),
                            ("population"               ,"abundant=a,clustered=c,numerous=n,scattered=s,several=v,solitary=y"),
                            ("habitat"                  ,"grasses=g,leaves=l,meadows=m,paths=p,urban=u,waste=w,woods=d"),
                        ];
        let mut result = Vec::new();
        for (facet, cats) in &facet_data {
            let mut facet_map = HashMap::new();
            for cat in cats.split(',') {
                let mut i = cat.splitn(2,'=');
                if let (Some(name),Some(c)) = (i.next(), i.next()) {
                    facet_map.insert(c.chars().next().unwrap(), name);
                } else {
                    panic!("Can't parse: {}", cat);
                }
            }
            result.push((*facet,facet_map));
        }
        result
    };
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
    assert_eq!(a.yes.rows.len(), 3);
    assert_eq!(a.no.rows.len(), 2);
}

#[derive(Debug)]
struct Answer {
    yes: Node,
    no: Node,
    impurity: f64,
    parent_idx: Option<usize>,
}

impl Answer {
    fn new(yes: Vec<Mush>, no: Vec<Mush>) -> Answer {
        let yes_node = Node::new(yes);
        let no_node = Node::new(no);
        let answer_impurity = yes_node.impurity + no_node.impurity;
        Answer {
            yes: yes_node,
            no: no_node,
            impurity: answer_impurity,
            parent_idx: None,
        }
    }
}

impl std::fmt::Display for Answer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "yes: {}, no {}, imp: {}",
            self.yes, self.no, self.impurity
        )
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
        Answer::new(poisons.clone(), edibles.clone()).impurity
            < Answer::new(poisons.clone(), mixed.clone()).impurity
    );
    assert!(
        (Answer::new(poisons.clone(), mixed.clone()).impurity
            - Answer::new(mixed.clone(), poisons.clone()).impurity)
            .abs()
            < std::f64::EPSILON
    );
    assert!(
        Answer::new(edibles.clone(), poisons.clone()).impurity
            < Answer::new(poisons.clone(), mixed.clone()).impurity
    );
}

#[derive(Debug)]
struct Node {
    rows: Vec<Mush>,
    poison_cnt: usize,
    impurity: f64,
}

impl Node {
    fn new(mushs: Vec<Mush>) -> Node {
        let gini = |poison_count, total_count| {
            let p = poison_count as f64;
            let l = total_count as f64;
            2.0 * (p / l) * ((l - p) / l)
        };

        let poison_cnt = mushs.iter().filter(|m| m.poisonous()).count();
        let impurity = gini(poison_cnt, mushs.len());
        Node {
            rows: mushs,
            poison_cnt,
            impurity,
        }
    }
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}/{} poisonous (imp {})",
            self.poison_cnt,
            self.rows.len(),
            self.impurity
        )
    }
}

const NUM_FACETS: usize = 22;

#[derive(Deserialize, Debug, Clone, Copy)]
struct Mush {
    poison: char,
    attrs: [char; NUM_FACETS],
}

impl Mush {
    fn poisonous(&self) -> bool {
        self.poison == 'p'
    }
}
