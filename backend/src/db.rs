use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use postgres_native_tls::MakeTlsConnector;
use serde::Serialize;
use tokio_postgres::Row;
use uuid::Uuid;

type ConnectionPool = Pool<PostgresConnectionManager<MakeTlsConnector>>;

#[derive(Debug, Serialize, Clone)]
pub struct Place {
    pub id: Uuid,
    pub name: String,
    pub maps_link: String,
}

pub trait PlacesDB {
    async fn get_place(&self, id: Uuid) -> Place;
    async fn list_places(&self) -> Vec<Place>;
    async fn create_place(&self, place: Place) -> Place;
}

#[derive(Clone)]
pub struct DB {
    pool: ConnectionPool,
}

pub fn new_db(pool: ConnectionPool) -> DB {
    return DB {
        pool
    };
}

fn row_to_place(r: &Row) -> Place {
    let id: Uuid = r.get("id");
    let name: String = r.get("name");
    let maps_link: String = r.get("maps_link");
    return Place {
        id,
        name,
        maps_link,
    };
}

impl PlacesDB for DB {
    async fn get_place(&self, id: Uuid) -> Place {
        let conn = self.pool.get().await.unwrap();
        let row = conn.query_one("SELECT * FROM places WHERE id = $1", &[&id]).await.unwrap();
        return row_to_place(&row);
    }

    async fn list_places(&self) -> Vec<Place> {
        let conn = self.pool.get().await.unwrap();
        let vec = conn.query("SELECT * FROM places", &[]).await.unwrap();
        return vec.iter()
            .map(row_to_place)
            .collect::<Vec<_>>();
    }

    async fn create_place(&self, place: Place) -> Place {
        let conn = self.pool.get().await.unwrap();
        conn.execute("INSERT INTO places (id, name, maps_link) VALUES ($1, $2, $3)",
                     &[&place.id, &place.name, &place.maps_link]).await.unwrap();
        return place;
    }
}