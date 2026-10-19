use hy_check::{check_file, CheckResult, Verdict};
use hy_diag::Severity;
use hy_syntax::parse_file;

fn check(src: &str) -> CheckResult {
    let ast = parse_file("test.hy", src).expect("test snippet must parse");
    check_file(&ast)
}

fn error_codes(r: &CheckResult) -> Vec<String> {
    r.diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| d.code.clone())
        .collect()
}

#[test]
fn rejects_and_accepts_generic_fn_constraints_729() {
    for src in [
        "fn add[t](a t, b t) t { return a + b }\nfn main() {}\n",
        "fn lt[t](a t, b t) bool { return a < b }\nfn main() {}\n",
        "fn neg[t](a t) t { return -a }\nfn main() {}\n",
        "fn inv[t](a t) bool { return !a }\nfn main() {}\n",
        "fn idx[t](a t) int { return a[0] }\nfn main() {}\n",
        "fn call_it[t](a t) { a() }\nfn main() {}\n",
        "fn bad_ret[t](a t) t { return 0 }\nfn main() {}\n",
        "fn bad_eq[t comparable](a t) bool { return a == 1 }\nfn main() {}\n",
        "fn same[t](a t, b t) t { return a }\nfn main() {\n    fmt.println(same(1, \"two\"))\n}\n",
        "fn id[t](x t) t { return x }\nfn main() {\n    fmt.println(id[int](\"wrong\"))\n}\n",
        "fn id[t](x t) t { return x }\nfn main() {\n    fmt.println(id(1, 2))\n}\n",
        "fn zero[t]() t {\n    var z t\n    return z\n}\nfn main() {\n    fmt.println(zero())\n}\n",
        "fn conv[t](x int) t { return t(x) }\nfn main() {}\n",
    ] {
        let r = check(src);
        assert_eq!(r.verdict, Verdict::Reject, "expected reject for:\n{src}");
        assert!(
            error_codes(&r).contains(&"HYB-E1002".to_string()),
            "expected HYB-E1002, got {:?} for:\n{src}",
            error_codes(&r)
        );
    }

    for src in [
        "fn eq[t](a t, b t) bool { return a == b }\nfn main() {}\n",
        "fn neq[t any](a t, b t) bool { return a != b }\nfn main() {}\n",
        "fn eq[t comparable](a t, b t) bool { return a == b }\nfn main() {\n    xs := []int{1}\n    fmt.println(eq(xs, xs))\n}\n",
        "fn eq[t comparable](a t, b t) bool { return a == b }\nfn main() {\n    xs := []int{1}\n    fmt.println(eq[[]int](xs, xs))\n}\n",
        "fn main() {\n    var s set[[]int]\n    fmt.println(len(s))\n}\n",
        "struct holder[k comparable] { m map[k]int }\nfn main() {\n    var h holder[[]int]\n    fmt.println(len(h.m))\n}\n",
    ] {
        let r = check(src);
        assert_eq!(r.verdict, Verdict::Reject, "expected reject for:\n{src}");
        assert!(
            error_codes(&r).contains(&"HYB-E2006".to_string()),
            "expected HYB-E2006, got {:?} for:\n{src}",
            error_codes(&r)
        );
    }

    let valid = "\
fn eq[t comparable](a t, b t) bool {
    return a == b
}

fn has_key[k, v](m map[k]v, key k) bool {
    _, ok := m[key]
    return ok
}

fn zero[t]() t {
    var z t
    return z
}

fn first[t](xs []t, fallback t) t {
    if len(xs) == 0 {
        return fallback
    }
    return xs[0]
}

fn main() {
    fmt.println(eq(1, 2))
    fmt.println(eq[string](\"a\", \"b\"))
    m := map[string]int{\"k\": 1}
    fmt.println(has_key(m, \"k\"))
    n := zero[int]()
    fmt.println(n + 1)
    xs := []int{10, 20}
    fmt.println(first(xs, 0) + 5)
}
";
    let r = check(valid);
    assert_eq!(
        r.verdict,
        Verdict::Accept,
        "unexpected diagnostics: {:?}",
        r.diagnostics
    );
}
