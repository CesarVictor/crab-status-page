function ferrisImg(status) {
  switch (status) {
    case 'up':      return '/img/ferris-happy.svg';
    case 'down':    return '/img/ferris-sad.svg';
    default:        return '/img/ferris-checking.svg';
  }
}

function ferrisClass(status) {
  switch (status) {
    case 'up':   return 'ferris-happy';
    case 'down': return 'ferris-sad';
    default:     return 'ferris-checking';
  }
}
