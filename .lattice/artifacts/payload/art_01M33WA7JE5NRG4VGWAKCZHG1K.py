from popquiz.bank import History
from popquiz.dedupe import check, record


def test_a_function_parameter_does_not_bind_a_separate_function_call():
    a = 'fn f(drop: i32) {} fn main() { drop(1); }'
    b = 'fn f(nope: i32) {} fn main() { nope(1); }'
    assert check('new', b, record(History(), 'old', a)).kind != 'normalized_duplicate'
