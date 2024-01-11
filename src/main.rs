use std::io::Read;
use std::ops::Range;
use anyhow::Result;

#[derive(Debug, PartialEq, Eq)]
struct IfChangeThenChange {
    if_change: (String, Range<usize>),
    then_change: Vec<String>,
}

// single-file format
// ---
// if-change
// lorem ipsum dolor
// sit amet
// then-change a/b/c.rs


// multi-file format
// ---
// if-change
// lorem ipsum dolor
// sit amet
// then-change
//   a/b/c.rs
//   a/b/c2.rs
//   a/b/c3.rs
// fi-change

impl IfChangeThenChange {
    // TODO: errors should hand back meaningful diagnostics about the malformed ictc
    fn from_str(path: &str, s: &str) -> Vec<IfChangeThenChange> {
        let mut ret: Vec<IfChangeThenChange> = Vec::new();
        let mut curr: Option<IfChangeThenChange> = None;
        let mut errors: Vec<String> = Vec::new();


        for (i, line ) in s.lines().enumerate() {
            println!("Parsing line {:?}", line);
            if line.starts_with("// if-change") {
                if let Some(ictc) = curr {
                    errors.push(format!("invalid if-change block starting on {:?}", ictc.if_change).to_string());
                }
                curr = Some(IfChangeThenChange {
                    if_change: (path.to_string(), i..i),
                    then_change: vec![],
                });

            } else if line.starts_with("// then-change") {
                if let Some(mut ictc) = curr {
                    let if_change_range = ictc.if_change.1;
                    ictc.if_change.1 = if_change_range.start..i;

                    // if this line is "then-change $filename"
                    if line.starts_with("// then-change ") {
                        // NB: little bit of a hack
                        ictc.then_change.push((&line["// then-change ".len()..]).to_string());
                        ret.push(ictc);
                        curr = None;
                    } else {
                        curr = Some(ictc);
                    }
                    // otherwise, we expect the next lines until fi-change to all
                    // be in the then_change set
                } else {
                    errors.push("no if-change preceding this then-change".to_string());
                }
            } else if line.starts_with("// fi-change") {
                match curr {
                    Some(ictc) => ret.push(ictc),
                    None => errors.push("fi-change found on line ??? but does not match a preceding then-change".to_string()),
                }
                curr = None;
            } else {
                if let Some(ictc) = &mut curr {
                    if !ictc.if_change.1.is_empty() {
                    ictc.then_change.push(line.to_string());
                    }
                }
            }
        }

        ret
    }
}

mod test {
    use crate::IfChangeThenChange;

    #[test]
    fn basic() -> anyhow::Result<()> {
        let parsed = IfChangeThenChange::from_str("if-change.foo", "\
lorem
// if-change
ipsum
dolor
sit
// then-change then-change.foo
amet");
        assert_eq!(parsed, vec![IfChangeThenChange {
            if_change: ("if-change.foo".to_string(), 1..5),
            then_change: vec!["then-change.foo".to_string()],

        }]);

        Ok(())
    }
}

fn main() -> Result<()> {

    // Create a mutable String to store the user input
    let mut input = String::new();


    // Read a line from stdin and store it in the 'input' String
    std::io::stdin().read_to_string(&mut input)
        .expect("Failed to read line");

    println!("stdin: {}", input);

    
    let mut patch = unidiff::PatchSet::new();
    patch.parse(input).ok().expect("Error parsing diff");

    for patched_file in patch {
        println!("patched file {}", patched_file.path());
        println!("diff says {} -> {}", patched_file.source_file, patched_file.target_file);
    }

    Ok(())
}

