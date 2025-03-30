use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    dtos::relation_tuple::{RelationTuple, Subject},
    errors::{HeimdallError, HeimdallResult},
};

pub fn build_insert(
    commit_time: DateTime<Utc>,
    nid: Uuid,
    relation_tuples: &[RelationTuple],
) -> HeimdallResult<(String, Vec<String>)> {
    if relation_tuples.is_empty() {
        return Err(HeimdallError::MalformedInput);
    }

    // PERF: rather than using a String here, we could use a Vec<&str> for optimization purposes?
    let mut query_builder = String::with_capacity(relation_tuples.len());
    let mut args: Vec<String> = Vec::with_capacity(relation_tuples.len() * 10);

    query_builder.
        push_str("INSERT INTO heimdall_relation_tuples (shard_id, nid, namespace, object, relation, subject_id, subject_set_namespace, subject_set_object, subject_set_relation, commit_time) VALUES ");

    for (i, relation_tuple) in relation_tuples.iter().enumerate() {
        let shard_id = Uuid::new_v4();
        let seperator = ", ";
        if i > 0 {
            query_builder.push_str(seperator);
        }
        let param_placeholder = format!(
            "${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}, ${}",
            (i * 10) + 1,
            (i * 10) + 2,
            (i * 10) + 3,
            (i * 10) + 4,
            (i * 10) + 5,
            (i * 10) + 6,
            (i * 10) + 7,
            (i * 10) + 8,
            (i * 10) + 9,
            (i * 10) + 10
        );

        query_builder
            .reserve(relation_tuples.len() * (param_placeholder.len() + 10 + seperator.len()));

        query_builder.push_str(&param_placeholder);

        let mut subject_id = String::new();
        let mut subject_set_namespace = String::new();
        let mut subject_set_object = String::new();
        let mut subject_set_relation = String::new();

        match &relation_tuple.subject {
            Subject::Direct(subject_direct) => {
                subject_id = subject_direct.id.to_string();
            }
            Subject::Set(subject_set) => {
                subject_set_namespace = subject_set.namespace.clone();
                subject_set_object = subject_set.object.to_string();
                subject_set_relation = subject_set.relation.clone();
            }
        }
        args.extend([
            shard_id.to_string(),
            nid.to_string(),
            relation_tuple.namespace.clone(),
            relation_tuple.object.to_string(),
            relation_tuple.relation.clone(),
            subject_id,
            subject_set_namespace,
            subject_set_object,
            subject_set_relation,
            commit_time.to_string(),
        ]);
    }
    Ok((query_builder, args))
}
