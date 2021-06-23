use super::*;
use mbus_api::{
    message_bus::v0::{BusError, MessageBus, MessageBusTrait},
    ReplyErrorKind, ResourceKind,
};
use types::v0::message_bus::mbus::{
    DestroyReplica, Filter, NodeId, PoolId, Replica, ReplicaId, ReplicaShareProtocol, ShareReplica,
    UnshareReplica,
};

pub(super) fn configure(cfg: &mut paperclip::actix::web::ServiceConfig) {
    cfg.service(get_replicas)
        .service(get_replica)
        .service(get_node_replicas)
        .service(get_node_pool_replicas)
        .service(get_node_pool_replica)
        .service(put_node_pool_replica)
        .service(put_pool_replica)
        .service(del_node_pool_replica)
        .service(del_pool_replica)
        .service(put_node_pool_replica_share)
        .service(put_pool_replica_share)
        .service(del_node_pool_replica_share)
        .service(del_pool_replica_share);
}

#[get("/replicas", tags(Replicas))]
async fn get_replicas() -> Result<Json<Vec<Replica>>, RestClusterError> {
    RestRespond::result(MessageBus::get_replicas(Filter::None).await)
        .map_err(RestClusterError::from)
}
#[get("/replicas/{id}", tags(Replicas))]
async fn get_replica(
    web::Path(replica_id): web::Path<ReplicaId>,
) -> Result<Json<Replica>, RestError> {
    RestRespond::result(MessageBus::get_replica(Filter::Replica(replica_id)).await)
}

#[get("/nodes/{id}/replicas", tags(Replicas))]
async fn get_node_replicas(
    web::Path(node_id): web::Path<NodeId>,
) -> Result<Json<Vec<Replica>>, RestError> {
    RestRespond::result(MessageBus::get_replicas(Filter::Node(node_id)).await)
}

#[get("/nodes/{node_id}/pools/{pool_id}/replicas", tags(Replicas))]
async fn get_node_pool_replicas(
    web::Path((node_id, pool_id)): web::Path<(NodeId, PoolId)>,
) -> Result<Json<Vec<Replica>>, RestError> {
    RestRespond::result(MessageBus::get_replicas(Filter::NodePool(node_id, pool_id)).await)
}
#[get(
    "/nodes/{node_id}/pools/{pool_id}/replicas/{replica_id}",
    tags(Replicas)
)]
async fn get_node_pool_replica(
    web::Path((node_id, pool_id, replica_id)): web::Path<(NodeId, PoolId, ReplicaId)>,
) -> Result<Json<Replica>, RestError> {
    RestRespond::result(
        MessageBus::get_replica(Filter::NodePoolReplica(node_id, pool_id, replica_id)).await,
    )
}

#[put(
    "/nodes/{node_id}/pools/{pool_id}/replicas/{replica_id}",
    tags(Replicas)
)]
async fn put_node_pool_replica(
    web::Path((node_id, pool_id, replica_id)): web::Path<(NodeId, PoolId, ReplicaId)>,
    create: web::Json<CreateReplicaBody>,
) -> Result<Json<Replica>, RestError> {
    put_replica(
        Filter::NodePoolReplica(node_id, pool_id, replica_id),
        create.into_inner(),
    )
    .await
}
#[put("/pools/{pool_id}/replicas/{replica_id}", tags(Replicas))]
async fn put_pool_replica(
    web::Path((pool_id, replica_id)): web::Path<(PoolId, ReplicaId)>,
    create: web::Json<CreateReplicaBody>,
) -> Result<Json<Replica>, RestError> {
    put_replica(
        Filter::PoolReplica(pool_id, replica_id),
        create.into_inner(),
    )
    .await
}

#[delete(
    "/nodes/{node_id}/pools/{pool_id}/replicas/{replica_id}",
    tags(Replicas)
)]
async fn del_node_pool_replica(
    web::Path((node_id, pool_id, replica_id)): web::Path<(NodeId, PoolId, ReplicaId)>,
) -> Result<JsonUnit, RestError> {
    destroy_replica(Filter::NodePoolReplica(node_id, pool_id, replica_id)).await
}
#[delete("/pools/{pool_id}/replicas/{replica_id}", tags(Replicas))]
async fn del_pool_replica(
    web::Path((pool_id, replica_id)): web::Path<(PoolId, ReplicaId)>,
) -> Result<JsonUnit, RestError> {
    destroy_replica(Filter::PoolReplica(pool_id, replica_id)).await
}

#[put(
    "/nodes/{node_id}/pools/{pool_id}/replicas/{replica_id}/share/{protocol}",
    tags(Replicas)
)]
async fn put_node_pool_replica_share(
    web::Path((node_id, pool_id, replica_id, protocol)): web::Path<(
        NodeId,
        PoolId,
        ReplicaId,
        ReplicaShareProtocol,
    )>,
) -> Result<Json<String>, RestError> {
    share_replica(
        Filter::NodePoolReplica(node_id, pool_id, replica_id),
        protocol,
    )
    .await
}
#[put(
    "/pools/{pool_id}/replicas/{replica_id}/share/{protocol}",
    tags(Replicas)
)]
async fn put_pool_replica_share(
    web::Path((pool_id, replica_id, protocol)): web::Path<(
        PoolId,
        ReplicaId,
        ReplicaShareProtocol,
    )>,
) -> Result<Json<String>, RestError> {
    share_replica(Filter::PoolReplica(pool_id, replica_id), protocol).await
}

#[delete(
    "/nodes/{node_id}/pools/{pool_id}/replicas/{replica_id}/share",
    tags(Replicas)
)]
async fn del_node_pool_replica_share(
    web::Path((node_id, pool_id, replica_id)): web::Path<(NodeId, PoolId, ReplicaId)>,
) -> Result<JsonUnit, RestError> {
    unshare_replica(Filter::NodePoolReplica(node_id, pool_id, replica_id)).await
}
#[delete("/pools/{pool_id}/replicas/{replica_id}/share", tags(Replicas))]
async fn del_pool_replica_share(
    web::Path((pool_id, replica_id)): web::Path<(PoolId, ReplicaId)>,
) -> Result<JsonUnit, RestError> {
    unshare_replica(Filter::PoolReplica(pool_id, replica_id)).await
}

async fn put_replica(filter: Filter, body: CreateReplicaBody) -> Result<Json<Replica>, RestError> {
    let create = match filter.clone() {
        Filter::NodePoolReplica(node_id, pool_id, replica_id) => {
            body.bus_request(node_id, pool_id, replica_id)
        }
        Filter::PoolReplica(pool_id, replica_id) => {
            let node_id = match MessageBus::get_pool(Filter::Pool(pool_id.clone())).await {
                Ok(replica) => replica.node,
                Err(error) => return Err(RestError::from(error)),
            };
            body.bus_request(node_id, pool_id, replica_id)
        }
        _ => {
            return Err(RestError::from(BusError {
                kind: ReplyErrorKind::Internal,
                resource: ResourceKind::Replica,
                source: "put_replica".to_string(),
                extra: "invalid filter for resource".to_string(),
            }))
        }
    };

    RestRespond::result(MessageBus::create_replica(create).await)
}

async fn destroy_replica(filter: Filter) -> Result<JsonUnit, RestError> {
    let destroy = match filter.clone() {
        Filter::NodePoolReplica(node_id, pool_id, replica_id) => DestroyReplica {
            node: node_id,
            pool: pool_id,
            uuid: replica_id,
        },
        Filter::PoolReplica(pool_id, replica_id) => {
            let node_id = match MessageBus::get_replica(filter).await {
                Ok(replica) => replica.node,
                Err(error) => return Err(RestError::from(error)),
            };

            DestroyReplica {
                node: node_id,
                pool: pool_id,
                uuid: replica_id,
            }
        }
        _ => {
            return Err(RestError::from(BusError {
                kind: ReplyErrorKind::Internal,
                resource: ResourceKind::Replica,
                source: "destroy_replica".to_string(),
                extra: "invalid filter for resource".to_string(),
            }))
        }
    };

    RestRespond::result(MessageBus::destroy_replica(destroy).await).map(JsonUnit::from)
}

async fn share_replica(
    filter: Filter,
    protocol: ReplicaShareProtocol,
) -> Result<Json<String>, RestError> {
    let share = match filter.clone() {
        Filter::NodePoolReplica(node_id, pool_id, replica_id) => ShareReplica {
            node: node_id,
            pool: pool_id,
            uuid: replica_id,
            protocol,
        },
        Filter::PoolReplica(pool_id, replica_id) => {
            let node_id = match MessageBus::get_replica(filter).await {
                Ok(replica) => replica.node,
                Err(error) => return Err(RestError::from(error)),
            };

            ShareReplica {
                node: node_id,
                pool: pool_id,
                uuid: replica_id,
                protocol,
            }
        }
        _ => {
            return Err(RestError::from(BusError {
                kind: ReplyErrorKind::Internal,
                resource: ResourceKind::Replica,
                source: "share_replica".to_string(),
                extra: "invalid filter for resource".to_string(),
            }))
        }
    };

    RestRespond::result(MessageBus::share_replica(share).await)
}

async fn unshare_replica(filter: Filter) -> Result<JsonUnit, RestError> {
    let unshare = match filter.clone() {
        Filter::NodePoolReplica(node_id, pool_id, replica_id) => UnshareReplica {
            node: node_id,
            pool: pool_id,
            uuid: replica_id,
        },
        Filter::PoolReplica(pool_id, replica_id) => {
            let node_id = match MessageBus::get_replica(filter).await {
                Ok(replica) => replica.node,
                Err(error) => return Err(RestError::from(error)),
            };

            UnshareReplica {
                node: node_id,
                pool: pool_id,
                uuid: replica_id,
            }
        }
        _ => {
            return Err(RestError::from(BusError {
                kind: ReplyErrorKind::Internal,
                resource: ResourceKind::Replica,
                source: "unshare_replica".to_string(),
                extra: "invalid filter for resource".to_string(),
            }))
        }
    };

    RestRespond::result(MessageBus::unshare_replica(unshare).await).map(JsonUnit::from)
}
