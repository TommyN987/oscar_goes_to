import { expect, it } from 'vitest';
import './layout.css';

type Rgba = [red: number, green: number, blue: number, alpha: number];

/** The color a set of utility classes paints as a background, in 0–255 channels as a canvas renders it. */
function paintedBackground(classes: string): Rgba {
	const swatch = document.createElement('div');
	swatch.className = classes;
	document.body.append(swatch);
	const background = getComputedStyle(swatch).backgroundColor;
	swatch.remove();

	const pixel = document.createElement('canvas').getContext('2d')!;
	pixel.fillStyle = background;
	pixel.fillRect(0, 0, 1, 1);
	const [red, green, blue, alpha] = pixel.getImageData(0, 0, 1, 1).data;
	return [red, green, blue, alpha];
}

it('rule 4.7.1: the dialog scrim is the neutral color at 60%', () => {
	const [neutralRed, neutralGreen, neutralBlue] = paintedBackground('bg-neutral');
	const [red, green, blue, alpha] = paintedBackground('bg-scrim');

	expect(alpha).toBe(Math.round(0.6 * 255));
	// A translucent pixel read back from a canvas is exact to one step per channel.
	expect(Math.abs(red - neutralRed)).toBeLessThanOrEqual(1);
	expect(Math.abs(green - neutralGreen)).toBeLessThanOrEqual(1);
	expect(Math.abs(blue - neutralBlue)).toBeLessThanOrEqual(1);
});

it("rule 4.7.1: Tailwind's own palette paints nothing, so the theme holds every color", () => {
	const transparent: Rgba = [0, 0, 0, 0];

	expect(paintedBackground('bg-red-500')).toEqual(transparent);
	expect(paintedBackground('bg-white')).toEqual(transparent);
});

it("rule 4.7.1: daisyUI's black, which tints its shadows, is the theme's ink", () => {
	expect(paintedBackground('bg-black')).toEqual(paintedBackground('bg-neutral'));
});

it('rule 4.7.1: a theme color still paints', () => {
	expect(paintedBackground('bg-neutral')[3]).toBe(255);
});
