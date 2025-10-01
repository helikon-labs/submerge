use dv_report_config::Config;
use dv_report_types::governance::polkassembly::{
    PolkassemblyReferendumComment, PolkassemblyReferendumCommentListResponse,
};
use dv_report_types::governance::subsquare::{
    SubsquarePagedData, SubsquareReferendum, SubsquareReferendumComment, SubsquareVoteCall,
};
use dv_report_types::substrate::network::Network;

pub struct SubsquareClient {
    http_client: reqwest::Client,
}

impl SubsquareClient {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        Ok(Self {
            http_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(
                    config.http.request_timeout_seconds,
                ))
                .build()?,
        })
    }

    pub async fn fetch_referendum(
        &self,
        network: &Network,
        index: u32,
    ) -> anyhow::Result<Option<SubsquareReferendum>> {
        let url = format!(
            "https://{}-api.subsquare.io/gov2/referendums/{index}?simple=false",
            network.chain,
        );
        let response = self.http_client.get(url).send().await?;
        if response.status().as_u16() == 404 {
            return Ok(None);
        }
        let referendum = response.json::<SubsquareReferendum>().await?;
        Ok(Some(referendum))
    }

    pub async fn fetch_referenda(
        &self,
        chain: &Network,
        page: u16,
        page_size: u16,
    ) -> anyhow::Result<SubsquarePagedData<SubsquareReferendum>> {
        let url = format!(
            "https://{}-api.subsquare.io/gov2/referendums?simple=false&page_size={page_size}&page={page}",
            chain.chain,
        );
        Ok(self
            .http_client
            .get(url)
            .send()
            .await?
            .json::<SubsquarePagedData<SubsquareReferendum>>()
            .await?)
    }

    pub async fn fetch_vote_calls(
        &self,
        chain: &Network,
        index: u32,
    ) -> anyhow::Result<Vec<SubsquareVoteCall>> {
        let url = format!(
            "https://{}-api.subsquare.io/gov2/referendums/{index}/vote-calls",
            chain.chain,
        );
        Ok(self
            .http_client
            .get(url)
            .send()
            .await?
            .json::<Vec<SubsquareVoteCall>>()
            .await?)
    }

    pub async fn fetch_subsquare_referendum_comments(
        &self,
        chain: &Network,
        index: u32,
    ) -> anyhow::Result<Vec<SubsquareReferendumComment>> {
        let mut comments = Vec::new();
        let page_size = 10;
        let mut page = 1;
        loop {
            let url = format!(
                "https://{}-api.subsquare.io/gov2/referendums/{index}/comments?page_size={page_size}&page={page}",
                chain.chain,
            );
            let data = self
                .http_client
                .get(url)
                .send()
                .await?
                .json::<SubsquarePagedData<SubsquareReferendumComment>>()
                .await?;
            if data.items.is_empty() {
                break;
            }
            data.items
                .iter()
                .cloned()
                .for_each(|item| comments.push(item));
            page += 1;
        }
        Ok(comments)
    }

    pub async fn fetch_polkassembly_referendum_comments(
        &self,
        chain: &Network,
        index: u32,
    ) -> anyhow::Result<Vec<PolkassemblyReferendumComment>> {
        let url = format!(
            "https://{}-api.subsquare.io/polkassembly-comments?post_id={}&post_type=ReferendumV2",
            chain.chain, index,
        );
        let data = self
            .http_client
            .get(url)
            .send()
            .await?
            .json::<PolkassemblyReferendumCommentListResponse>()
            .await?;
        Ok(data
            .comments
            .iter()
            .filter(|c| c.comment_source == "polkassembly")
            .cloned()
            .collect())
    }
}
