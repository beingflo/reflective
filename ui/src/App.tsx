import { Route, Router } from '@solidjs/router';
import { type Component } from 'solid-js';
import Login from './pages/Login';
import View from './pages/View';
import Upload from './pages/Upload';
import NotFound from './pages/NotFound';

const App: Component = () => {
  return (
    <Router>
      <Route path="/" component={View} />
      <Route path="/login" component={Login} />
      <Route path="/upload" component={Upload} />
      <Route path="*404" component={NotFound} />
    </Router>
  );
};

export default App;
