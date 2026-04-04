#[allow(clippy::needless_range_loop)]
pub fn levenshtein(a: &str, b: &str) -> usize {
    let m = a.len(); let n = b.len();
    let mut dp = vec![vec![0usize; n+1]; m+1];
    for i in 0..=m { dp[i][0] = i; }
    for j in 0..=n { dp[0][j] = j; }
    for i in 1..=m { for j in 1..=n {
        let cost = if a.as_bytes()[i-1] == b.as_bytes()[j-1] { 0 } else { 1 };
        dp[i][j] = (dp[i-1][j]+1).min(dp[i][j-1]+1).min(dp[i-1][j-1]+cost);
    }}
    dp[m][n]
}

pub fn suggest_variable(name: &str, available: &[String]) -> Option<String> {
    available.iter()
        .filter(|s| levenshtein(name, s) <= 2)
        .min_by_key(|s| levenshtein(name, s))
        .cloned()
}
