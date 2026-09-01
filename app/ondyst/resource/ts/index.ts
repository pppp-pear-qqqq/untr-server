// const title = document.querySelector('h1')!;
const clock_hand = document.querySelector('#clock-hand')!;

function updateClock() {
	const now = new Date();
	const sec = now.getHours() * 60 + now.getMinutes();
	const angle = sec * 360 / 1440;
	clock_hand.setAttribute('transform', `rotate(${angle},0,0)`);
}

setInterval(updateClock, 60000);
updateClock();
