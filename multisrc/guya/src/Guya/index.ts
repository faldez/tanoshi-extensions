import { GuyaBase } from '../guyabase';

export default class Guya extends GuyaBase {
    id: number = 7;
    name: string = "Guya";
    url: string = "https://guya.moe";
    version: string = "0.1.0";
    icon: string = "https://guya.moe/static/logo_small.png";
    languages: string = "end";
    nsfw: boolean = false;
}