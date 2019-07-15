use std::collections::{HashMap, HashSet};
use std::iter::once;

use crate::asm::{Label, Op, OpArg};
use crate::token::{IntoTokens, Tokens};

#[derive(Clone, Debug)]
pub struct LabelTokens(pub String, pub Tokens);

pub type LabelMap<'a> = HashMap<String, &'a Label>;
pub type AddressMap = HashMap<String, u8>;

pub fn assemble(mut labels: Vec<Label>) -> Result<Vec<u8>, String> {
    weak_optimize_ops(&mut labels);

    remove_unused_labels(&mut labels);

    optimize_tail_jumps(&mut labels);

    let label_tokens = get_label_tokens(labels)?;

    let address_map = get_address_map(&label_tokens)?;

    get_bytes(&label_tokens, &address_map)
}

fn get_label_tokens(labels: Vec<Label>) -> Result<Vec<LabelTokens>, String> {
    labels
        .into_iter()
        .map(|label| (label.0.clone(), label.into_tokens()))
        .map(|(name, res)| res.map(|tokens| LabelTokens(name, tokens)))
        .collect()
}

fn get_label_map(labels: &[Label]) -> LabelMap {
    labels
        .iter()
        .map(|label| (label.0.clone(), label))
        .collect()
}

fn get_address_map(labels: &[LabelTokens]) -> Result<AddressMap, String> {
    let label_sizes = labels.iter().map(|LabelTokens(_, tokens)| tokens.len());

    let high_address: usize = label_sizes.clone().sum();

    if high_address <= 256 {
        let addresses = label_sizes
            .scan(0, |addr, size| {
                *addr += size;
                Some(*addr)
            })
            .map(|addr| addr as u8);
        let address_map = labels
            .iter()
            .map(|LabelTokens(name, _)| name.clone())
            .zip(once(0u8).chain(addresses))
            .collect();
        Ok(address_map)
    } else {
        Err(format!(
            "program too large ({} bytes > {} bytes)",
            high_address, 256
        ))
    }
}

fn get_bytes(labels: &[LabelTokens], address_map: &AddressMap) -> Result<Vec<u8>, String> {
    let bytes = labels
        .iter()
        .map(|label_token| label_token.as_bytes(address_map))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(bytes.into_iter().flatten().collect())
}

fn remove_unused_labels(labels: &mut Vec<Label>) {
    let label_map = get_label_map(&labels);

    let mut label_names: HashSet<String> = HashSet::new();

    fn add_referenced_labels<'a>(
        label: &'a Label,
        label_map: &LabelMap<'a>,
        label_names: &mut HashSet<String>,
    ) {
        let (name, ops) = (label.0.clone(), &label.1);

        label_names.insert(name);

        let directly_referenced_labels = ops
            .iter()
            .flat_map(|op| {
                op.1.iter().filter_map(|arg| match arg {
                    OpArg::Ident(name) => Some(name.clone()),
                    _ => None,
                })
            })
            .filter(|name| !label_names.contains(name))
            .filter_map(|name| label_map.get(&name))
            .collect::<Vec<_>>();

        for label in directly_referenced_labels {
            add_referenced_labels(label, &label_map, label_names);
        }
    }

    if let Some(label) = labels.first() {
        add_referenced_labels(&label, &label_map, &mut label_names);
        labels.retain(|label| label_names.contains(&label.0))
    }
}

fn optimize_tail_jumps(labels: &mut Vec<Label>) {
    let mut label_iter = labels.iter_mut().peekable();

    while let (Some(label), Some(next)) = (label_iter.next(), label_iter.peek()) {
        if let Some(Op(op, args)) = label.1.last() {
            if op.as_str() == "jmp" {
                if let Some(OpArg::Ident(dest)) = args.first() {
                    if dest == &next.0 {
                        label.1.pop();
                    }
                }
            }
        }
    }
}

fn weak_optimize_ops(labels: &mut Vec<Label>) {
    fn is_op0_redundant(first: &Op, second: &Op) -> bool {
        match (first.0.as_str(), second.0.as_str()) {
            ("load", "load") => true,
            ("cjmp", "jmp") | ("icjmp", "jmp") => first.1 == second.1,
            _ => false,
        }
    }

    fn is_op1_redundant(first: &Op, second: &Op) -> bool {
        match (first.0.as_str(), second.0.as_str()) {
            ("store", "store") | ("store", "load") | ("load", "store") => first.1 == second.1,
            _ => false,
        }
    }

    fn optimize_label_ops(ops: &mut Vec<Op>) {
        if let Some((index, _)) = ops
            .iter()
            .enumerate()
            .find(|(_, op)| op.0.as_str() == "jmp" || op.0.as_str() == "halt")
        {
            ops.truncate(index + 1);
        }
        ops.dedup_by(|op0, op1| is_op1_redundant(op0, op1));
        ops.reverse();
        ops.dedup_by(|op0, op1| is_op0_redundant(op0, op1));
        ops.reverse();
    }

    for label in labels {
        optimize_label_ops(&mut label.1);
    }
}

impl LabelTokens {
    pub fn as_bytes(&self, address_map: &AddressMap) -> Result<Vec<u8>, String> {
        self.1
            .iter()
            .map(|token| token.as_byte(address_map))
            .collect()
    }
}
