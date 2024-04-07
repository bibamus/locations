use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use log::info;
use native_tls::TlsConnector;
use postgres_native_tls::MakeTlsConnector;
use serde::Serialize;
use tokio_postgres::{Config, Row};
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
    async fn update_place(&self, place: Place) -> Place;
    async fn delete_place(&self, id: Uuid);
}

#[derive(Clone)]
pub struct DB {
    pool: ConnectionPool,
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

impl DB {
    pub async fn new_db(config: Config) -> DB {
        let connector = TlsConnector::builder()
            .build()
            .unwrap();
        let connector = MakeTlsConnector::new(connector);
        let manager = PostgresConnectionManager::new(config, connector);
        let pool = Pool::builder().build(manager).await.unwrap();
        return DB {
            pool
        };
    }
    pub async fn init_db(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Initializing Database");
        let conn = self.pool.get().await?;
        conn.batch_execute("CREATE TABLE IF NOT EXISTS \
         places \
         (id UUID, \
          name VARCHAR NOT NULL, \
          maps_link VARCHAR NOT NULL)").await?;
        info!("Database initialised");
        Ok(())
    }
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

    async fn update_place(&self, place: Place) -> Place {
        let conn = self.pool.get().await.unwrap();
        conn.execute("UPDATE places SET name = $2, maps_link = $3 WHERE id = $1",
                     &[&place.id, &place.name, &place.maps_link]).await.unwrap();
        return place;
    }

    async fn delete_place(&self, id: Uuid) {
        let conn = self.pool.get().await.unwrap();
        conn.execute("DELETE FROM places WHERE id = $1", &[&id]).await.unwrap();
    }
}