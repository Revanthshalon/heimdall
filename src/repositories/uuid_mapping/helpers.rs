use crate::dtos::uuid_mapping::UuidMappings;

pub fn build_insert_uuids(uuid_mappings: &[UuidMappings]) -> (String, Vec<String>) {
    if uuid_mappings.is_empty() {
        return (String::new(), Vec::new());
    }

    let mut query_builder = String::new();
    let mut args: Vec<String> = Vec::with_capacity(uuid_mappings.len() * 2);

    query_builder
        .push_str("INSERT INTO heimdall_uuid_mappings (id, string_representation) VALUES ");

    for (i, uuid_mapping) in uuid_mappings.iter().enumerate() {
        if i > 0 {
            query_builder.push_str(", ");
        }
        let param_placeholder = format!("(${}, ${})", (i * 2) + 1, (i * 2) + 2);
        query_builder.push_str(&param_placeholder);
        args.push(uuid_mapping.id.to_string());
        args.push(uuid_mapping.string_representation.clone())
    }

    query_builder.push_str("ON CONFLICT (id) DO NOTHING");

    (query_builder, args)
}
