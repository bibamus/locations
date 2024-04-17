import {getToken} from "@/auth/auth"

const baseurl = import.meta.env.VITE_API_BASEURL ?? "";


export interface PlaceWithRating {
    place: Place;
    average_rating: number;
    own_rating: number;
}

export interface Place {
    id: string;
    name: string;
    maps_link: string;
}

export async function getPlaces(): Promise<PlaceWithRating[]> {
    const token = await getToken();
    const response = await fetch(`${baseurl}/place`, {
        method: "GET",
        headers: {
            "Content-Type": "application/json",
            "Authorization": `Bearer ${token}`,
        },
    });
    if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
    }
    return response.json();
}

export async function createPlace(place: Place): Promise<void> {
    const token = await getToken();
    const response = await fetch(`${baseurl}/place`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
            "Authorization": `Bearer ${token}`,
        },
        body: JSON.stringify(place),
    });
    if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
    }
}

export async function updatePlace(place: Place): Promise<void> {
    const token = await getToken();
    const response = await fetch(`${baseurl}/place/${place.id}`, {
        method: "PUT",
        headers: {
            "Content-Type": "application/json",
            "Authorization": `Bearer ${token}`,
        },
        body: JSON.stringify(place),
    });
    if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
    }
}

export async function ratePlace(placeId: string, rating: number): Promise<void> {
    const token = await getToken();
    const response = await fetch(`${baseurl}/place/${placeId}/rating`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
            "Authorization": `Bearer ${token}`,
        },
        body: JSON.stringify(rating),
    });
    if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
    }
}