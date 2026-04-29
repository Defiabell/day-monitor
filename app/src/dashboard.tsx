import React from 'react';
import ReactDOM from 'react-dom/client';
import { Dashboard } from './dashboard/Dashboard';
import './styles.css';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <Dashboard />
  </React.StrictMode>,
);
