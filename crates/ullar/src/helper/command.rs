#[macro_export]
macro_rules! parse_override_args {
    ($cmd:ident, $params: ident) => {
        $params.split_whitespace().for_each(|param| {
            $cmd.arg(param);
        })
    };
}
