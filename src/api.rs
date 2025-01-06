use anyhow::Result;
use quick_xml::de::from_str;
use serde::Deserialize;
use std::time::Duration;
use ureq::Agent;

#[derive(Deserialize, Debug)]
pub struct Ranking {
    #[serde(rename = "NAME")]
    pub nation: String,
    #[serde(rename = "RANK")]
    pub rank: u64,
    #[serde(rename = "SCORE")]
    pub score: f64,
}

#[derive(Deserialize, Debug)]
struct Nation {
    #[serde(rename = "NATION")]
    nation: Vec<Ranking>,
}

#[derive(Deserialize, Debug)]
struct Region {
    #[serde(rename = "NATIONS")]
    nations: Nation,
}

#[derive(Deserialize, Debug)]
struct APIResponse {
    #[serde(rename = "CENSUSRANKS")]
    region: Region,
}

pub struct CensusReader {
    agent: Agent,
}

impl CensusReader {
    // Create CensusReader with an already-primed user-agent, so we can reuse the one from earlier
    pub fn with_agent(agent: &Agent) -> Self {
        Self {
            agent: agent.clone(),
        }
    }

    fn get_page(&self, region: &String, census_id: &u8, start: &u64) -> Result<Vec<Ranking>> {
        let url = format!("https://www.nationstates.net/cgi-bin/api.cgi?region={region}&q=censusranks;scale={census_id};start={start}");
        let response = &self.agent.get(&url).call()?.into_string()?;
        let response: APIResponse = from_str(&response)?;
        Ok(response.region.nations.nation)
    }

    pub fn get_rankings(&self, region: &String, census_id: &u8) -> Result<Vec<Ranking>> {
        let mut rankings = Vec::new();
        let mut prev_highest_ranking: u64;
        let mut highest_ranking: u64 = 0;

        loop {
            if let Ok(page_rankings) = self.get_page(&region, &census_id, &(highest_ranking + 1)) {
                // Add the rankings
                for ranking in page_rankings {
                    rankings.push(ranking);
                }

                prev_highest_ranking = highest_ranking;
                highest_ranking = rankings.iter().max_by_key(|x| x.rank).unwrap().rank; // There will always
                                                                                        // be at least one
                                                                                        // No change to the highest ranking, meaning no new nations
                if prev_highest_ranking == highest_ranking {
                    break;
                }

                // Rate limit sleep
                std::thread::sleep(Duration::from_secs(1));
            } else {
                // Parsing broke, must have reached the end lmao
                break;
            }
        }

        // Technically not required, but why not?
        rankings.sort_by(|x, y| x.score.partial_cmp(&y.score).unwrap());

        Ok(rankings)
    }
}
