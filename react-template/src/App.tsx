import React from 'react';
import ReactDOM from 'react-dom/client';

function App(props: {}) {
  return <>Hello, world!</>;
}

const rootDiv = document.getElementById('root')!;
ReactDOM.createRoot(rootDiv).render(<App />);