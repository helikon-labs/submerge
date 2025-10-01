use crate::postgres::PostgreSQLStorage;
use dv_report_types::governance::polkassembly::PolkassemblyReferendumComment;
use dv_report_types::governance::subsquare::SubsquareReferendumComment;
use dv_report_types::substrate::account_id::AccountId;
use std::str::FromStr;

impl PostgreSQLStorage {
    pub async fn save_subsquare_referendum_comment(
        &self,
        network_id: u32,
        referendum_index: u32,
        comment: &SubsquareReferendumComment,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
            INSERT INTO subsquare_referendum_comment(id, network_id, referendum_index, referendum_post_id, reply_to_comment_id, content, content_type, content_version, author_username, author_public_key, author_address, author_email_md5, height, created_at, updated_at, data_source, cid, proposer)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
            ON CONFLICT(id) DO UPDATE
            SET
                network_id = EXCLUDED.network_id,
                referendum_index = EXCLUDED.referendum_index,
                referendum_post_id = EXCLUDED.referendum_post_id,
                reply_to_comment_id = EXCLUDED.reply_to_comment_id,
                content = EXCLUDED.content,
                content_type = EXCLUDED.content_type,
                content_version = EXCLUDED.content_version,
                author_username = EXCLUDED.author_username,
                author_public_key = EXCLUDED.author_public_key,
                author_address = EXCLUDED.author_address,
                author_email_md5 = EXCLUDED.author_email_md5,
                height = EXCLUDED.height,
                created_at = EXCLUDED.created_at,
                updated_at = EXCLUDED.updated_at,
                data_source = EXCLUDED.data_source,
                cid = EXCLUDED.cid,
                proposer = EXCLUDED.proposer
            "#,
        )
            .bind(&comment.id)
            .bind(network_id as i32)
            .bind(referendum_index as i32)
            .bind(&comment.referendum_post_id)
            .bind(&comment.reply_to_comment_id)
            .bind(&comment.content)
            .bind(&comment.content_type)
            .bind(&comment.content_version)
            .bind(&comment.author.username)
            .bind(comment.author.public_key.map(|pk| pk.to_string()))
            .bind(comment.author.address.to_string())
            .bind(&comment.author.email_md5)
            .bind(comment.height as i32)
            .bind(comment.created_at)
            .bind(comment.updated_at)
            .bind(&comment.data_source)
            .bind(&comment.cid)
            .bind(comment.proposer.to_string())
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }

    pub async fn save_polkassembly_referendum_comment(
        &self,
        network_id: u32,
        referendum_index: u32,
        comment: &PolkassemblyReferendumComment,
        reply_to_comment_id: Option<&str>,
    ) -> anyhow::Result<()> {
        let proposer = if let Some(proposer) = comment.proposer.as_deref() {
            AccountId::from_str(proposer).ok()
        } else {
            None
        };
        sqlx::query(
            r#"
            INSERT INTO polkassembly_referendum_comment(id, network_id, referendum_index, content, updated_at, created_at, proposer, username, reply_to_comment_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            ON CONFLICT(id) DO UPDATE
            SET
                network_id = EXCLUDED.network_id,
                referendum_index = EXCLUDED.referendum_index,
                content = EXCLUDED.content,
                updated_at = EXCLUDED.updated_at,
                created_at = EXCLUDED.created_at,
                proposer = EXCLUDED.proposer,
                username = EXCLUDED.username,
                reply_to_comment_id = EXCLUDED.reply_to_comment_id
            "#,
        )
            .bind(&comment.id)
            .bind(network_id as i32)
            .bind(referendum_index as i32)
            .bind(&comment.content)
            .bind(comment.updated_at)
            .bind(comment.created_at)
            .bind(proposer.map(|p| p.to_string()))
            .bind(comment.username.to_string())
            .bind(reply_to_comment_id)
            .execute(&self.connection_pool)
            .await?;
        Ok(())
    }
}
