//! Github stargazers example

#[tokio::test(flavor = "multi_thread", worker_threads = 10)]
async fn test_github_data() {
    use crate::dtkutils::dtk_reqwest::send_get_request;
    use crate::dtkutils::dtk_reqwest::validate_response;
    use crate::dtkutils::dtk_github::get_all_stargazers;
    use crate::dtkutils::dtk_github::GithubData;
    let request_url = format!(
        "https://api.github.com/repos/{owner}/{repo}/stargazers",
        owner = "solana-labs",
        repo = "solana"
    );
    let response = match send_get_request(&request_url).await {
        Ok(r) => r,
        Err(err) => {
            println!("Request failed: {}", err.to_string());
            return;
        }
    };
    if validate_response(&response).is_some() {
        match response.json::<Vec<GithubData>>().await {
            Ok(users) => {
                println!("Users: {}", users.len());
                // assert!(users.len() > 0);
            }
            Err(error) => panic!("json deserialization failed {:?}", error),
        };
    }
    let github_data = get_all_stargazers("baakeydow", "flipper0-rust-hello-world").await;
    if github_data.is_ok() {
        let data = github_data.unwrap();
        println!("Repo data: {:#?}", data.len());
    } else {
        println!("Error");
    }
}
