import { DateTime } from 'luxon';

export function capitalize(str) {
    if (!str) return str;
    return str.charAt(0).toUpperCase() + str.slice(1);
}

export function formatDate(date, format, defaultValue) {
    let val = defaultValue
    if (date) {
        format = format || 'EEE dd MMM'
        val = DateTime.fromJSDate(new Date(date)).toFormat(format)
    }
    return val
}

export function steamOgImage(appid) {
    return `https://shared.akamai.steamstatic.com/store_item_assets/steam/apps/${appid}/capsule_sm_120.jpg`
}

export function parseColor(color) {
    if (!color) return
    if (color.startsWith('#')) {
        return color;
    } else if (color.startsWith('var(--')) {
        return color;
    } else if (color === 'inherit') {
        return 'inherit';
    } else return `var(--pico-color-${color})`;
}

export function flagFromCountryCode(countryCode) {
    const codePoints = countryCode
        .toUpperCase()
        .split('')
        .map(char => 127397 + char.charCodeAt());
    return String.fromCodePoint(...codePoints);
}