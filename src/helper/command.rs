#[macro_export]
macro_rules! parse_optional_params {
    ($cmd:ident, $params: ident) => {
        $params.split_whitespace().for_each(|param| {
            $cmd.arg(param);
        });
    };
}
