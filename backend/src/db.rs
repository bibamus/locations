use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use log::info;
use native_tls::TlsConnector;
use postgres_native_tls::MakeTlsConnector;
use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive;
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

#[derive(Debug, Serialize, Clone)]
pub struct PlaceWithRating {
    pub place: Place,
    pub average_rating: f64,
    pub own_rating: i32,
}

impl Place {
    pub(crate) fn new(name: String, maps_link: String) -> Place {
        return Place {
            id: Uuid::new_v4(),
            name,
            maps_link,
        };
    }
}

pub struct Rating {
    pub place_id: Uuid,
    pub user_id: Uuid,
    pub rating: i32,
}

pub trait PlacesDB {
    async fn get_place(&self, id: Uuid) -> Place;
    async fn list_places(&self) -> Vec<Place>;
    async fn create_place(&self, place: Place) -> Place;
    async fn update_place(&self, place: Place) -> Place;
    async fn delete_place(&self, id: Uuid);
    async fn rate_place(&self, place_id: Uuid, user_id: String, rating: i32) -> Place;
    async fn get_place_with_rating(&self, id: Uuid, user_id: String) -> PlaceWithRating;
    async fn list_places_with_rating(&self, user_id: String) -> Vec<PlaceWithRating>;
}

#[derive(Clone)]
pub struct DB {
    pool: ConnectionPool,
}


fn row_to_place(r: &Row) -> Place {
    let id: Uuid = r.get("id");
    let name: String = r.get("name");
    let maps_link: String = r.get("maps_link");
    // let average_rating: Option<Decimal> = r.get("average_rating");
    // let own_rating: Option<i32> = r.get("rating");
    return Place {
        id,
        name,
        maps_link,
        // average_rating: average_rating.map(|d| d.to_f64().unwrap()).unwrap_or(0.0),
        // own_rating: own_rating.unwrap_or(0),
    };
}

fn row_to_place_with_rating(r: &Row) -> PlaceWithRating {
    let place = row_to_place(r);
    let average_rating: Option<Decimal> = r.get("average_rating");
    let own_rating: Option<i32> = r.get("rating");
    return PlaceWithRating {
        place,
        average_rating: average_rating.map(|d| d.to_f64().unwrap()).unwrap_or(0.0),
        own_rating: own_rating.unwrap_or(0),
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
          maps_link VARCHAR NOT NULL,\
          PRIMARY KEY (id) \
          )").await?;

        conn.batch_execute("CREATE TABLE IF NOT EXISTS \
            ratings \
            (place_id UUID, \
            user_id VARCHAR NOT NULL, \
            rating INT NOT NULL,\
            PRIMARY KEY (place_id, user_id) \
            )").await?;
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
        let vec = conn.query("SELECT * FROM places ORDER BY name ASC
        ", &[]).await.unwrap();
        return vec.iter()
            .map(row_to_place)
            .collect::<Vec<_>>();
    }

    async fn get_place_with_rating(&self, id: Uuid, user_id: String) -> PlaceWithRating {
        let conn = self.pool.get().await.unwrap();
        let row = conn.query_one("SELECT * FROM places LEFT JOIN \
        (SELECT place_id, rating FROM ratings WHERE user_id = $2) AS own_rating on places.id = own_rating.place_id \
        LEFT JOIN \
            (SELECT place_id, AVG(rating) as average_rating FROM ratings GROUP BY place_id) \
            AS ratings ON places.id = ratings.place_id
         WHERE id = $1", &[&id, &user_id]).await.unwrap();
        return row_to_place_with_rating(&row);
    }

    async fn list_places_with_rating(&self, user_id: String) -> Vec<PlaceWithRating> {
        let conn = self.pool.get().await.unwrap();
        let vec = conn.query("SELECT * FROM places LEFT JOIN \
        (SELECT place_id, rating FROM ratings WHERE user_id = $1) AS own_rating on places.id = own_rating.place_id \
         LEFT JOIN \
            (SELECT place_id, AVG(rating) as average_rating FROM ratings GROUP BY place_id) \
            AS ratings ON places.id = ratings.place_id
         ORDER BY name ASC
        ", &[&user_id]).await.unwrap();
        return vec.iter()
            .map(row_to_place_with_rating)
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

    async fn rate_place(&self, place_id: Uuid, user_id: String, rating: i32) -> Place {
        let conn = self.pool.get().await.unwrap();
        conn.execute("INSERT INTO ratings (place_id, user_id, rating) VALUES ($1, $2, $3) \
                      ON CONFLICT (place_id, user_id) DO UPDATE SET rating = $3",
                     &[&place_id, &user_id, &rating]).await.unwrap();
        return self.get_place(place_id).await;
    }
}