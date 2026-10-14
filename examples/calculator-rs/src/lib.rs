//! Example plugin: a calculator page.
//!
//! The page shows the number being typed, or the last result, with the
//! pending operation above it. Buttons and the keyboard do the same things:
//! the page declares the keys it handles, and a click and a key press both
//! become one action name. The copy button puts the shown number on the
//! clipboard, the one permission the plugin declares. The state survives a
//! restart: every render saves it, and `open` gets it back.
//!
//! The TypeScript twin in `examples/calculator-ts` draws the same tree for
//! the same input; the tests compare the two node by node. Both round
//! results the same way (see `format_number`), so they show the same digits.

use tessoa_plugin_sdk::pages::*;
use tessoa_plugin_sdk::prelude::*;

/// Keys the page handles while it has the focus.
const KEYS: &[&str] = &[
    "0",
    "1",
    "2",
    "3",
    "4",
    "5",
    "6",
    "7",
    "8",
    "9",
    ".",
    "+",
    "-",
    "*",
    "/",
    "=",
    "Enter",
    "Backspace",
    "Escape",
    "Delete",
    "Ctrl+c",
    "Meta+c",
];

/// Longest number the user can type, in characters.
const MAX_DIGITS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

impl Op {
    fn symbol(self) -> &'static str {
        match self {
            Op::Add => "+",
            Op::Sub => "−",
            Op::Mul => "×",
            Op::Div => "÷",
        }
    }

    /// How the operation is saved.
    fn code(self) -> &'static str {
        match self {
            Op::Add => "+",
            Op::Sub => "-",
            Op::Mul => "*",
            Op::Div => "/",
        }
    }

    fn from_code(code: &str) -> Option<Op> {
        Some(match code {
            "+" => Op::Add,
            "-" => Op::Sub,
            "*" => Op::Mul,
            "/" => Op::Div,
            _ => return None,
        })
    }

    fn apply(self, a: f64, b: f64) -> f64 {
        match self {
            Op::Add => a + b,
            Op::Sub => a - b,
            Op::Mul => a * b,
            Op::Div => a / b,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Calculator {
    /// The number shown: what the user is typing, a result, or `Error`.
    display: String,
    /// The left operand of the pending operation.
    acc: Option<f64>,
    op: Option<Op>,
    /// The next digit starts a new number instead of extending this one.
    fresh: bool,
}

/// A result as the display shows it: rounded to ten decimals, without
/// trailing zeros, `Error` when it is not a number or too large. The rounding
/// is done by hand so that Rust and TypeScript print the same digits: both
/// round the same doubles in the same way, and the fixed-point formatting of
/// the rounded value is then exact in both.
fn format_number(x: f64) -> String {
    if !x.is_finite() || x.abs() >= 1e15 {
        return "Error".to_string();
    }
    let r = (x.abs() * 1e10).round() / 1e10;
    let mut s = format!("{:.10}", if x < 0.0 { -r } else { r });
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    if s == "-0" {
        s = "0".to_string();
    }
    s
}

/// The action of a declared key, named like the button that does the same.
fn key_action(key: &str) -> Option<String> {
    Some(
        match key {
            "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" => return Some(format!("k{key}")),
            "." => "dot",
            "+" => "add",
            "-" => "sub",
            "*" => "mul",
            "/" => "div",
            "=" | "Enter" => "eq",
            "Backspace" => "back",
            "Escape" | "Delete" => "clear",
            "Ctrl+c" | "Meta+c" => "copy",
            _ => return None,
        }
        .to_string(),
    )
}

impl Calculator {
    fn new() -> Calculator {
        Calculator {
            display: "0".to_string(),
            acc: None,
            op: None,
            fresh: true,
        }
    }

    /// The saved state: four lines, display, operand, operation and the
    /// fresh flag.
    fn encode(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}",
            self.display,
            self.acc.map(|a| a.to_string()).unwrap_or_default(),
            self.op.map(Op::code).unwrap_or(""),
            u8::from(self.fresh)
        )
    }

    fn decode(state: &[u8]) -> Option<Calculator> {
        let text = std::str::from_utf8(state).ok()?;
        let mut lines = text.split('\n');
        let display = lines.next()?.to_string();
        let acc = match lines.next()? {
            "" => None,
            a => Some(a.parse().ok()?),
        };
        let op = match lines.next()? {
            "" => None,
            o => Some(Op::from_code(o)?),
        };
        let fresh = lines.next()? == "1";
        Some(Calculator {
            display,
            acc,
            op,
            fresh,
        })
    }

    /// The number shown, or `None` for `Error`.
    fn value(&self) -> Option<f64> {
        self.display.parse().ok()
    }

    fn pending(&self) -> String {
        match (self.acc, self.op) {
            (Some(acc), Some(op)) => format!("{} {}", format_number(acc), op.symbol()),
            _ => String::new(),
        }
    }

    fn digit(&mut self, d: &str) {
        if self.fresh || self.display == "0" || self.display == "Error" {
            self.display = d.to_string();
        } else if self.display.trim_start_matches('-').len() < MAX_DIGITS {
            self.display.push_str(d);
        }
        self.fresh = false;
    }

    fn dot(&mut self) {
        if self.fresh || self.display == "Error" {
            self.display = "0.".to_string();
        } else if !self.display.contains('.') && self.display.trim_start_matches('-').len() < MAX_DIGITS {
            self.display.push('.');
        }
        self.fresh = false;
    }

    fn fail(&mut self) {
        *self = Calculator {
            display: "Error".to_string(),
            ..Calculator::new()
        };
    }

    fn operator(&mut self, op: Op) {
        let Some(value) = self.value() else { return };
        match (self.acc, self.op) {
            (Some(acc), Some(pending)) if !self.fresh => {
                let r = pending.apply(acc, value);
                self.display = format_number(r);
                if self.display == "Error" {
                    return self.fail();
                }
                self.acc = Some(r);
            }
            (Some(_), Some(_)) => {}
            _ => self.acc = Some(value),
        }
        self.op = Some(op);
        self.fresh = true;
    }

    fn equals(&mut self) {
        let (Some(acc), Some(op), Some(value)) = (self.acc, self.op, self.value()) else {
            self.fresh = true;
            return;
        };
        self.display = format_number(op.apply(acc, value));
        if self.display == "Error" {
            return self.fail();
        }
        self.acc = None;
        self.op = None;
        self.fresh = true;
    }

    fn back(&mut self) {
        if self.fresh || self.display == "Error" {
            return;
        }
        self.display.pop();
        if self.display.is_empty() || self.display == "-" {
            self.display = "0".to_string();
        }
    }

    /// Do what the button with this key does.
    fn press(&mut self, action: &str) -> Result<(), String> {
        match action {
            "clear" => *self = Calculator::new(),
            "back" => self.back(),
            "dot" => self.dot(),
            "add" => self.operator(Op::Add),
            "sub" => self.operator(Op::Sub),
            "mul" => self.operator(Op::Mul),
            "div" => self.operator(Op::Div),
            "eq" => self.equals(),
            "copy" => {
                clipboard::write_text(&self.display).map_err(|e| format!("cannot copy the result: {e:?}"))?;
            }
            _ => match action.strip_prefix('k') {
                Some(d) if d.len() == 1 && d.as_bytes()[0].is_ascii_digit() => self.digit(d),
                _ => return Err(format!("no action named {action:?}")),
            },
        }
        Ok(())
    }
}

impl Page for Calculator {
    fn open(_page: PageId, state: Option<Vec<u8>>, _args: Option<Vec<u8>>) -> Self {
        state
            .as_deref()
            .and_then(Calculator::decode)
            .unwrap_or_else(Calculator::new)
    }

    fn event(&mut self, event: Event) -> Result<(), String> {
        match event {
            Event::Click { key } => self.press(&key),
            Event::Key(k) => match key_action(&k.key) {
                Some(action) => self.press(&action),
                None => Ok(()),
            },
            _ => Ok(()),
        }
    }

    fn render(&self) -> Node {
        let key = |key: &str, label: &str| btn(label).key(key).grow();
        col([
            text(self.pending())
                .key("pending")
                .size(TextSize::Caption)
                .tone(Tone::Dim)
                .align(Align::End),
            text(&self.display)
                .key("display")
                .size(TextSize::Display)
                .align(Align::End),
            row([key("clear", "C"), key("back", "⌫"), key("div", "÷"), key("mul", "×")]).gap(Space::S),
            row([key("k7", "7"), key("k8", "8"), key("k9", "9"), key("sub", "−")]).gap(Space::S),
            row([key("k4", "4"), key("k5", "5"), key("k6", "6"), key("add", "+")]).gap(Space::S),
            row([
                key("k1", "1"),
                key("k2", "2"),
                key("k3", "3"),
                key("eq", "=").kind(ButtonKind::Primary),
            ])
            .gap(Space::S),
            row([key("k0", "0"), key("dot", "."), key("copy", "Copy")]).gap(Space::S),
        ])
        .pad(Space::M)
        .gap(Space::S)
    }

    fn keys(&self) -> Vec<String> {
        KEYS.iter().map(|k| k.to_string()).collect()
    }

    fn save(&self) -> Option<Vec<u8>> {
        Some(self.encode().into_bytes())
    }
}

tessoa_plugin_sdk::export_pages!(Calculator => "calculator");

#[cfg(test)]
mod tests {
    use super::*;

    fn run(actions: &[&str]) -> Calculator {
        let mut c = Calculator::new();
        for a in actions {
            c.press(a).unwrap();
        }
        c
    }

    #[test]
    fn numbers_are_rounded_to_ten_decimals() {
        assert_eq!(format_number(0.1 + 0.2), "0.3");
        assert_eq!(format_number(1.0 / 3.0), "0.3333333333");
        assert_eq!(format_number(2.0 / 3.0), "0.6666666667");
        assert_eq!(format_number(-2.5), "-2.5");
        assert_eq!(format_number(-1e-12), "0");
        assert_eq!(format_number(1e14), "100000000000000");
        assert_eq!(format_number(1e15), "Error");
        assert_eq!(format_number(f64::INFINITY), "Error");
        assert_eq!(format_number(f64::NAN), "Error");
    }

    #[test]
    fn typing_and_operations() {
        assert_eq!(run(&["k1", "k2", "add", "k3", "eq"]).display, "15");
        assert_eq!(
            run(&["k1", "add", "k2", "mul", "k3", "eq"]).display,
            "9",
            "left to right"
        );
        let c = run(&["k7", "sub"]);
        assert_eq!((c.display.as_str(), c.pending().as_str()), ("7", "7 −"));
        assert_eq!(run(&["k1", "div", "k0", "eq"]).display, "Error");
        assert_eq!(
            run(&["k1", "div", "k0", "eq", "k5"]).display,
            "5",
            "a digit starts over"
        );
        assert_eq!(run(&["dot", "k5", "dot", "k1"]).display, "0.51");
        assert_eq!(run(&["k1", "k2", "back", "back", "back"]).display, "0");
        assert_eq!(
            run(&["k9", "add", "mul", "k2", "eq"]).display,
            "18",
            "the last operator counts"
        );
    }

    #[test]
    fn the_state_survives_a_round_trip() {
        let c = run(&["k1", "dot", "k5", "mul"]);
        assert_eq!(Calculator::decode(c.encode().as_bytes()), Some(c));
        assert_eq!(Calculator::decode(b"garbage"), None);
    }
}
