#![allow(
    dead_code,
    no_self_field_access,
    private_single_call_fn,
    statement_group_spacing,
    struct_invariant_note
)]

struct Fields(u8, u8, u8, u8, u8);

macro_rules! generated_destructure {
    ($value:expr) => {
        let Self(_, _, _, _, _) = $value;
    };
}

macro_rules! wildcard {
    () => {
        _
    };
}

impl Fields {
    fn leading(&self) {
        let Self(_, _, a, b, c) = self;
        let _ = (a, b, c);
    }

    fn trailing(&self) {
        let Self(a, b, c, _, _) = self;
        let _ = (a, b, c);
    }

    fn all_wildcards(self) {
        let Self(_, _, _, _, _) = self;
    }

    fn both_edges(&self) {
        let Self(_, _, middle, _, _) = self;
        let _ = middle;
    }

    fn shared_reference(&self) {
        let &Self(_, _, a, b, c) = self;
        let _ = (a, b, c);
    }

    fn mutable_reference(&mut self) {
        let &mut Self(a, b, c, _, _) = self;
        let _ = (a, b, c);
    }

    fn other_value(&self, other: &Self) {
        let Self(_, _, a, b, c) = other;
        let _ = (a, b, c);
    }

    fn commented(&self) {
        let Self(_, /* preserve this comment */ _, a, b, c) = self;
        let _ = (a, b, c);
    }

    fn accepted(&self) {
        let Self(_, a, b, c, _) = self;
        let _ = (a, b, c);

        let Self(a, _, _, b, c) = self;
        let _ = (a, b, c);

        let Self(_a, _b, a, b, c) = self;
        let _ = (a, b, c);

        let Self(a @ _, b @ _, c, d, e) = self;
        let _ = (a, b, c, d, e);

        let Self(.., a, b, c) = self;
        let _ = (a, b, c);

        let Self(a, b, c, ..) = self;
        let _ = (a, b, c);

        let Self(..) = self;

        // A second rest pattern would be invalid, even if the first matches no fields.
        let Self(_, _, .., a, b, c) = self;
        let _ = (a, b, c);

        let Self(a, b, c, .., _, _) = self;
        let _ = (a, b, c);

        let Fields(_, _, a, b, c) = self;
        let _ = (a, b, c);

        let (_, _, a) = (1, 2, 3);
        let _ = a;

        let [_, _, a] = [1, 2, 3];
        let _ = a;

        // Match arms are outside the scope of local let patterns.
        match self {
            Self(_, _, a, b, c) => {
                let _ = (a, b, c);
            }
        }

        generated_destructure!(self);

        // Do not report a run that contains a macro-generated wildcard.
        let Self(wildcard!(), _, a, b, c) = self;
        let _ = (a, b, c);
    }

    #[allow(self_pattern_rest)]
    fn explicitly_allowed(&self) {
        let Self(_, _, a, b, c) = self;
        let _ = (a, b, c);
    }
}

struct Single(u8, u8);

impl Single {
    fn accepted(&self) {
        let Self(_, value) = self;
        let _ = value;
    }
}

struct Named {
    first: u8,
    second: u8,
}

impl Named {
    fn accepted(&self) {
        let Self { first: _, second: _ } = self;
    }
}

fn main() {}
