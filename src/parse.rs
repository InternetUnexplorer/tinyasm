use nom::branch::alt;
use nom::bytes::complete::{escaped_transform, is_a, is_not, tag};

use nom::character::complete::alpha1;
use nom::character::complete::not_line_ending;
use nom::character::complete::oct_digit1;
use nom::character::complete::{alphanumeric1, newline};
use nom::character::complete::{digit1, space0};
use nom::character::complete::{hex_digit1, space1};
use nom::combinator::{all_consuming, map, map_res, not, opt, peek, recognize};

use nom::multi::{many0, many1, separated_list, separated_nonempty_list};
use nom::sequence::{delimited, pair, preceded, terminated, tuple};
use nom::IResult;

use crate::asm::{Label, Op, OpArg};

fn ident(input: &str) -> IResult<&str, &str> {
    let head = alt((alpha1, tag("_")));
    let body = alt((alphanumeric1, tag("_")));
    recognize(tuple((head, many0(body))))(input)
}

fn number(input: &str) -> IResult<&str, usize> {
    let hex_number = map_res(preceded(tag("0x"), hex_digit1), |s: &str| {
        usize::from_str_radix(s, 16)
    });
    let oct_number = map_res(preceded(tag("0o"), oct_digit1), |s: &str| {
        usize::from_str_radix(s, 8)
    });
    let bin_number = map_res(preceded(tag("0b"), is_a("01")), |s: &str| {
        usize::from_str_radix(s, 2)
    });
    let dec_number = map_res(digit1, |s: &str| s.parse());

    alt((hex_number, oct_number, bin_number, dec_number))(input)
}

fn string(input: &str) -> IResult<&str, String> {
    let string_escape = alt((
        map(tag("\\"), |_| "\\"),
        map(tag("\""), |_| "\""),
        map(tag("\n"), |_| ""),
        map(tag("t"), |_| "\t"),
        map(tag("r"), |_| "\r"),
        map(tag("n"), |_| "\n"),
    ));

    let string_inner = escaped_transform(is_not("\\\""), '\\', string_escape);

    delimited(tag("\""), string_inner, tag("\""))(input)
}

fn comment(input: &str) -> IResult<&str, ()> {
    map(preceded(tag(";"), not_line_ending), |_| ())(input)
}

fn whitespace(input: &str) -> IResult<&str, ()> {
    map(space1, |_| ())(input)
}

fn transparent(input: &str) -> IResult<&str, ()> {
    map(many1(alt((whitespace, comment))), |_| ())(input)
}

fn transparent_line(input: &str) -> IResult<&str, ()> {
    map(terminated(opt(transparent), newline), |_| ())(input)
}

fn transparent_lines(input: &str) -> IResult<&str, ()> {
    map(many1(transparent_line), |_| ())(input)
}

fn op_arg(input: &str) -> IResult<&str, OpArg> {
    let ident_op_arg = map(ident, |a| OpArg::Ident(a.into()));
    let number_op_arg = map(number, OpArg::Number);
    let string_op_arg = map(string, OpArg::String);

    alt((ident_op_arg, number_op_arg, string_op_arg))(input)
}

fn op(input: &str) -> IResult<&str, Op> {
    let op_name = recognize(delimited(opt(tag(".")), alpha1, peek(not(tag(":")))));
    let op_arg_sep = delimited(space0, tag(","), space0);
    let op_args = preceded(space0, separated_list(op_arg_sep, op_arg));

    map(pair(op_name, op_args), |(name, args)| {
        Op(name.to_ascii_lowercase(), args)
    })(input)
}

fn label(input: &str) -> IResult<&str, Label> {
    let label_name = terminated(ident, tag(":"));
    let label_op = preceded(space0, op);
    let label_ops = preceded(
        opt(transparent_lines),
        separated_nonempty_list(transparent_lines, label_op),
    );

    map(pair(label_name, label_ops), |(name, ops)| {
        Label(name.into(), ops)
    })(input)
}

pub fn parse(input: &str) -> IResult<&str, Vec<Label>> {
    let labels = many0(delimited(
        opt(transparent_lines),
        label,
        opt(transparent_lines),
    ));
    all_consuming(labels)(input)
}
