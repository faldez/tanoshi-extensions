export interface Detail {
    author: string,
    artist: string,
    description: string,
    slug: string,
    cover: string,
    groups: {
        [key: string]: string
    },
    last_updated: number
}

export interface Series {
    slug: string,
    title: string,
    description: string,
    author: string,
    artist: string,
    groups: {
        [key: string]: string
    },
    cover: string,
    preferred_sort: string[],
    chapters: {
        [key: string]: Chapter
    },
    next_release_page: boolean,
    next_release_time: number,
    next_release_html: string,
}

export interface Chapter {
    volume: string,
    title: string,
    folder: string,
    groups: {
        [key: string]: string[]
    },
    release_date: {
        [key: string]: number
    },
}

export interface Response {
    [title: string]: Detail
}