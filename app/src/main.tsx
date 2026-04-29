import React from 'react';
import ReactDOM from 'react-dom/client';
import { Popover } from './popover/Popover';
import './styles.css';

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <Popover />
  </React.StrictMode>,
);
