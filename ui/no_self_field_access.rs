#![allow(dead_code, statement_group_spacing)]

struct Gpr;

// NOTE(invariant): The wrapped register is intentionally opaque for the fixture.
struct Wrapper(Gpr);

impl Wrapper {
    fn allowed(&self) -> &Gpr {
        let Self(value) = self;

        value
    }
}

// NOTE(invariant): Register storage is intentionally opaque for the fixture.
struct Registers {
    rdx: Gpr,
    rsi: Gpr,
    rdi: Gpr,
}

impl Registers {
    fn forbidden_shared(&self) -> &Gpr {
        &self.rsi
    }

    fn forbidden_mut(&mut self) -> &mut Gpr {
        &mut self.rdi
    }

    fn forbidden_owned(self) -> Gpr {
        self.rdx
    }

    fn forbidden_deref(&mut self) -> &mut Gpr {
        &mut (*self).rsi
    }

    fn forbidden_alias(&self) -> &Gpr {
        let this = self;

        &this.rsi
    }

    fn forbidden_alias_destructure(&self) -> &Gpr {
        let this = self;
        let Self { rsi, .. } = this;

        rsi
    }

    fn forbidden_match_destructure(&self) -> &Gpr {
        match self {
            Self { rsi, .. } => rsi,
        }
    }

    fn forbidden_late_destructure(&self) -> &Gpr {
        let _marker = 0_u8;
        let Self { rsi, .. } = self;

        rsi
    }

    fn forbidden_nested_destructure(&self) -> &Gpr {
        {
            let Self { rsi, .. } = self;

            rsi
        }
    }

    async fn forbidden_async_late(&self) -> &Gpr {
        let _marker = 0_u8;
        let Self { rsi, .. } = self;

        rsi
    }

    fn allowed_shared(&self) -> &Gpr {
        let Self { rsi, .. } = self;

        rsi
    }

    fn allowed_explicit_ref(&self) -> &Gpr {
        let &Self { ref rsi, .. } = self;

        rsi
    }

    async fn allowed_async(&self) -> &Gpr {
        let Self { rsi, .. } = self;

        rsi
    }

    fn allowed_mut(&mut self) -> &mut Gpr {
        let Self { rdi, .. } = self;

        rdi
    }

    fn allowed_owned(self) -> Gpr {
        let Self { rdx, .. } = self;

        rdx
    }

    fn allowed_call(&self) -> &Gpr {
        self.allowed_shared()
    }

    fn allowed_other_binding(&self, other: &Self) {
        let _ = &other.rdx;
        let _ = self.allowed_shared();
    }
}

fn main() {}
