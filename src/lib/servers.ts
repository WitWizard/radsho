import type { Server, ServerCategory } from "../types/vpn";

export const SERVERS: Server[] = [
  { id: "firenze", city: "Firenze", cityFa: "فلورانس", tag: "IT", region: "IT", flag: "IT", ping: 69, colo: "FLR", tags: ["fast", "streaming", "antisanction"] },
  { id: "milan", city: "Milan", cityFa: "میلان", tag: "IT", region: "IT", flag: "IT", ping: 66, colo: "MXP", tags: ["fast", "gaming", "antisanction"] },
  { id: "rome", city: "Rome", cityFa: "رم", tag: "IT", region: "IT", flag: "IT", ping: 72, colo: "FCO", tags: ["streaming", "antisanction"] },
  { id: "frankfurt", city: "Frankfurt", cityFa: "فرانکفورت", tag: "DE", region: "DE", flag: "DE", ping: 64, colo: "FRA", tags: ["fast", "gaming", "antisanction", "doublehop"] },
  { id: "berlin", city: "Berlin", cityFa: "برلین", tag: "DE", region: "DE", flag: "DE", ping: 71, colo: "TXL", tags: ["fast", "gaming", "antisanction"] },
  { id: "munich", city: "Munich", cityFa: "مونیخ", tag: "DE", region: "DE", flag: "DE", ping: 68, colo: "MUC", tags: ["fast", "streaming", "antisanction"] },
  { id: "amsterdam", city: "Amsterdam", cityFa: "آمستردام", tag: "NL", region: "NL", flag: "NL", ping: 70, colo: "AMS", tags: ["fast", "streaming", "gaming", "antisanction"] },
  { id: "rotterdam", city: "Rotterdam", cityFa: "روتردام", tag: "NL", region: "NL", flag: "NL", ping: 73, colo: "RTM", tags: ["fast", "doublehop"] },
  { id: "vienna", city: "Vienna", cityFa: "وین", tag: "AT", region: "AT", flag: "AT", ping: 67, colo: "VIE", tags: ["fast", "antisanction", "doublehop"] },
  { id: "zurich", city: "Zurich", cityFa: "زوریخ", tag: "CH", region: "CH", flag: "CH", ping: 71, colo: "ZRH", tags: ["fast", "antisanction", "doublehop"] },
  { id: "geneva", city: "Geneva", cityFa: "ژنو", tag: "CH", region: "CH", flag: "CH", ping: 75, colo: "GVA", tags: ["doublehop", "antisanction"] },
  { id: "paris", city: "Paris", cityFa: "پاریس", tag: "FR", region: "FR", flag: "FR", ping: 78, colo: "CDG", tags: ["fast", "streaming", "gaming", "antisanction"] },
  { id: "marseille", city: "Marseille", cityFa: "مارسی", tag: "FR", region: "FR", flag: "FR", ping: 82, colo: "MRS", tags: ["streaming", "antisanction"] },
  { id: "madrid", city: "Madrid", cityFa: "مادرید", tag: "ES", region: "ES", flag: "ES", ping: 88, colo: "MAD", tags: ["fast", "streaming", "gaming"] },
  { id: "barcelona", city: "Barcelona", cityFa: "بارسلونا", tag: "ES", region: "ES", flag: "ES", ping: 85, colo: "BCN", tags: ["streaming", "gaming"] },
  { id: "london", city: "London", cityFa: "لندن", tag: "GB", region: "GB", flag: "GB", ping: 81, colo: "LHR", tags: ["fast", "streaming", "gaming", "antisanction"] },
  { id: "manchester", city: "Manchester", cityFa: "منچستر", tag: "GB", region: "GB", flag: "GB", ping: 86, colo: "MAN", tags: ["streaming", "gaming"] },
  { id: "brussels", city: "Brussels", cityFa: "بروکسل", tag: "BE", region: "BE", flag: "BE", ping: 74, colo: "BRU", tags: ["fast", "antisanction"] },
  { id: "dublin", city: "Dublin", cityFa: "دوبلین", tag: "IE", region: "IE", flag: "IE", ping: 91, colo: "DUB", tags: ["streaming", "gaming"] },
  { id: "lisbon", city: "Lisbon", cityFa: "لیسبون", tag: "PT", region: "PT", flag: "PT", ping: 96, colo: "LIS", tags: ["streaming"] },
  { id: "stockholm", city: "Stockholm", cityFa: "استکهلم", tag: "SE", region: "SE", flag: "SE", ping: 84, colo: "ARN", tags: ["fast", "gaming", "antisanction", "doublehop"] },
  { id: "oslo", city: "Oslo", cityFa: "اسلو", tag: "NO", region: "NO", flag: "NO", ping: 89, colo: "OSL", tags: ["antisanction", "doublehop"] },
  { id: "helsinki", city: "Helsinki", cityFa: "هلسینکی", tag: "FI", region: "FI", flag: "FI", ping: 86, colo: "HEL", tags: ["fast", "gaming", "antisanction"] },
  { id: "copenhagen", city: "Copenhagen", cityFa: "کپنهاگ", tag: "DK", region: "DK", flag: "DK", ping: 80, colo: "CPH", tags: ["fast", "antisanction"] },
  { id: "reykjavik", city: "Reykjavik", cityFa: "ریکیاویک", tag: "IS", region: "IS", flag: "IS", ping: 112, colo: "KEF", tags: ["doublehop", "antisanction"] },
  { id: "warsaw", city: "Warsaw", cityFa: "ورشو", tag: "PL", region: "PL", flag: "PL", ping: 77, colo: "WAW", tags: ["fast", "gaming", "antisanction"] },
  { id: "prague", city: "Prague", cityFa: "پراگ", tag: "CZ", region: "CZ", flag: "CZ", ping: 74, colo: "PRG", tags: ["fast", "gaming", "doublehop"] },
  { id: "bratislava", city: "Bratislava", cityFa: "براتیسلاوا", tag: "SK", region: "SK", flag: "SK", ping: 76, colo: "BTS", tags: ["doublehop", "antisanction"] },
  { id: "budapest", city: "Budapest", cityFa: "بوداپست", tag: "HU", region: "HU", flag: "HU", ping: 75, colo: "BUD", tags: ["doublehop", "gaming"] },
  { id: "bucharest", city: "Bucharest", cityFa: "بخارست", tag: "RO", region: "RO", flag: "RO", ping: 79, colo: "OTP", tags: ["fast", "doublehop", "antisanction"] },
  { id: "sofia", city: "Sofia", cityFa: "صوفیه", tag: "BG", region: "BG", flag: "BG", ping: 78, colo: "SOF", tags: ["doublehop", "gaming"] },
  { id: "athens", city: "Athens", cityFa: "آتن", tag: "GR", region: "GR", flag: "GR", ping: 83, colo: "ATH", tags: ["streaming", "gaming"] },
  { id: "belgrade", city: "Belgrade", cityFa: "بلگراد", tag: "RS", region: "RS", flag: "RS", ping: 82, colo: "BEG", tags: ["doublehop"] },
  { id: "zagreb", city: "Zagreb", cityFa: "زاگرب", tag: "HR", region: "HR", flag: "HR", ping: 77, colo: "ZAG", tags: ["doublehop"] },
  { id: "ljubljana", city: "Ljubljana", cityFa: "لیوبلیانا", tag: "SI", region: "SI", flag: "SI", ping: 76, colo: "LJU", tags: ["doublehop"] },
  { id: "tallinn", city: "Tallinn", cityFa: "تالین", tag: "EE", region: "EE", flag: "EE", ping: 88, colo: "TLL", tags: ["antisanction", "doublehop"] },
  { id: "riga", city: "Riga", cityFa: "ریگا", tag: "LV", region: "LV", flag: "LV", ping: 87, colo: "RIX", tags: ["antisanction", "doublehop"] },
  { id: "vilnius", city: "Vilnius", cityFa: "ویلنیوس", tag: "LT", region: "LT", flag: "LT", ping: 86, colo: "VNO", tags: ["antisanction", "doublehop"] },
  { id: "kyiv", city: "Kyiv", cityFa: "کی‌یف", tag: "UA", region: "UA", flag: "UA", ping: 94, colo: "KBP", tags: ["doublehop"] },
  { id: "chisinau", city: "Chisinau", cityFa: "کیشیناو", tag: "MD", region: "MD", flag: "MD", ping: 89, colo: "RMO", tags: ["doublehop"] },
  { id: "istanbul", city: "Istanbul", cityFa: "استانبول", tag: "TR", region: "TR", flag: "TR", ping: 54, colo: "IST", tags: ["fast", "gaming", "streaming"] },
  { id: "dubai", city: "Dubai", cityFa: "دبی", tag: "AE", region: "AE", flag: "AE", ping: 49, colo: "DXB", tags: ["fast", "gaming"] },
  { id: "newyork", city: "New York", cityFa: "نیویورک", tag: "US", region: "US", flag: "US", ping: 148, colo: "EWR", tags: ["fast", "streaming", "antisanction"] },
  { id: "losangeles", city: "Los Angeles", cityFa: "لس‌آنجلس", tag: "US", region: "US", flag: "US", ping: 186, colo: "LAX", tags: ["fast", "gaming", "streaming"] },
  { id: "miami", city: "Miami", cityFa: "میامی", tag: "US", region: "US", flag: "US", ping: 162, colo: "MIA", tags: ["streaming", "gaming"] },
  { id: "chicago", city: "Chicago", cityFa: "شیکاگو", tag: "US", region: "US", flag: "US", ping: 158, colo: "ORD", tags: ["gaming", "antisanction"] },
  { id: "dallas", city: "Dallas", cityFa: "دالاس", tag: "US", region: "US", flag: "US", ping: 171, colo: "DFW", tags: ["streaming", "antisanction"] },
  { id: "seattle", city: "Seattle", cityFa: "سیاتل", tag: "US", region: "US", flag: "US", ping: 182, colo: "SEA", tags: ["gaming", "antisanction"] },
  { id: "ashburn", city: "Ashburn", cityFa: "اشبرن", tag: "US", region: "US", flag: "US", ping: 145, colo: "IAD", tags: ["fast", "antisanction", "doublehop"] },
  { id: "toronto", city: "Toronto", cityFa: "تورنتو", tag: "CA", region: "CA", flag: "CA", ping: 154, colo: "YYZ", tags: ["fast", "streaming", "antisanction"] },
  { id: "montreal", city: "Montreal", cityFa: "مونترال", tag: "CA", region: "CA", flag: "CA", ping: 149, colo: "YUL", tags: ["streaming", "doublehop"] },
  { id: "vancouver", city: "Vancouver", cityFa: "ونکوور", tag: "CA", region: "CA", flag: "CA", ping: 179, colo: "YVR", tags: ["gaming", "streaming"] },
  { id: "saopaulo", city: "Sao Paulo", cityFa: "سائوپائولو", tag: "BR", region: "BR", flag: "BR", ping: 210, colo: "GRU", tags: ["streaming", "gaming"] },
  { id: "kingston", city: "Kingston", cityFa: "کینگستون", tag: "JM", region: "JM", flag: "JM", ping: 195, colo: "KIN", tags: ["streaming"] },
  { id: "singapore", city: "Singapore", cityFa: "سنگاپور", tag: "SG", region: "SG", flag: "SG", ping: 118, colo: "SIN", tags: ["fast", "gaming", "antisanction"] },
  { id: "tokyo", city: "Tokyo", cityFa: "توکیو", tag: "JP", region: "JP", flag: "JP", ping: 152, colo: "NRT", tags: ["fast", "gaming", "streaming", "antisanction"] },
  { id: "osaka", city: "Osaka", cityFa: "اوساکا", tag: "JP", region: "JP", flag: "JP", ping: 156, colo: "KIX", tags: ["gaming", "streaming"] },
  { id: "seoul", city: "Seoul", cityFa: "سئول", tag: "KR", region: "KR", flag: "KR", ping: 164, colo: "ICN", tags: ["gaming", "streaming"] },
  { id: "hongkong", city: "Hong Kong", cityFa: "هنگ‌کنگ", tag: "HK", region: "HK", flag: "HK", ping: 138, colo: "HKG", tags: ["fast", "gaming"] },
  { id: "taipei", city: "Taipei", cityFa: "تایپه", tag: "TW", region: "TW", flag: "TW", ping: 146, colo: "TPE", tags: ["gaming", "antisanction"] },
  { id: "mumbai", city: "Mumbai", cityFa: "بمبئی", tag: "IN", region: "IN", flag: "IN", ping: 98, colo: "BOM", tags: ["streaming", "gaming"] },
  { id: "sydney", city: "Sydney", cityFa: "سیدنی", tag: "AU", region: "AU", flag: "AU", ping: 218, colo: "SYD", tags: ["streaming", "antisanction"] },
  { id: "auckland", city: "Auckland", cityFa: "اوکلند", tag: "NZ", region: "NZ", flag: "NZ", ping: 232, colo: "AKL", tags: ["streaming"] },
  { id: "johannesburg", city: "Johannesburg", cityFa: "ژوهانسبورگ", tag: "ZA", region: "ZA", flag: "ZA", ping: 174, colo: "JNB", tags: ["gaming"] },
];

export const FAST_QUICK_IDS: readonly string[] = ["losangeles", "madrid", "frankfurt", "amsterdam", "london", "newyork"];

export const CATEGORY_ORDER: readonly ServerCategory[] = ["fast", "streaming", "gaming", "antisanction", "doublehop"];

export function findServer(id: string): Server {
  return SERVERS.find((server) => server.id === id) ?? SERVERS[0];
}

export function serverByRegion(region: string, preferId: string): Server {
  const preferred = SERVERS.find((server) => server.id === preferId && server.region === region);
  return preferred ?? SERVERS.find((server) => server.region === region) ?? findServer(preferId);
}

export function serverLabel(server: Server, lang: "en" | "fa"): string {
  return `${lang === "fa" ? server.cityFa : server.city}, ${server.tag}`;
}

export function fakeIp(seed: string): string {
  let hash = 2166136261;
  for (let i = 0; i < seed.length; i += 1) {
    hash ^= seed.charCodeAt(i);
    hash = Math.imul(hash, 16777619) >>> 0;
  }
  const a = 80 + (hash % 120);
  const b = (hash >>> 8) % 250;
  const c = (hash >>> 16) % 250;
  const d = 2 + ((hash >>> 24) % 250);
  return `${a}.${b}.${c}.${d}`;
}