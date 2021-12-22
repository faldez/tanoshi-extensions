export interface Title {
    english: string;
    japanese: string;
    pretty: string;
}

export interface Page {
    t: string;
    w: number;
    h: number;
}

export interface Cover {
    t: string;
    w: number;
    h: number;
}

export interface Thumbnail {
    t: string;
    w: number;
    h: number;
}

export interface Images {
    pages: Page[];
    cover: Cover;
    thumbnail: Thumbnail;
}

export interface Tag {
    id: number;
    type: string;
    name: string;
    url: string;
    count: number;
}

export interface Result {
    id: any;
    media_id: string;
    title: Title;
    images: Images;
    scanlator: string;
    upload_date: number;
    tags: Tag[];
    num_pages: number;
    num_favorites: number;
}

export interface Response {
    result: Result[];
    num_pages: number;
    per_page: number;
}