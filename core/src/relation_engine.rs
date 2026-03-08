use crate::schema::{Entity, Relation};

pub fn infer_relations(entities: &[Entity]) -> Vec<Relation> {
    let mut routes = Vec::new();
    let mut tables = Vec::new();
    let mut configs = Vec::new();

    for entity in entities {
        match entity.entity_type.as_str() {
            "http_route" => routes.push(entity),
            "db_table" => tables.push(entity),
            "env_var_use" => configs.push(entity),
            _ => {}
        }
    }

    let mut relations = Vec::new();

    for route in &routes {
        for table in &tables {
            relations.push(Relation {
                record_type: "relation".into(),
                relation_type: "may_write_to".into(),
                src_entity: route.id.clone(),
                dst_entity: table.id.clone(),
            });
        }
        for cfg in &configs {
            relations.push(Relation {
                record_type: "relation".into(),
                relation_type: "configured_by".into(),
                src_entity: route.id.clone(),
                dst_entity: cfg.id.clone(),
            });
        }
    }

    relations
}
