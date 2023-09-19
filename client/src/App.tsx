import { QueryClient, QueryClientProvider } from 'react-query'
import './App.css'
import { Clients } from './Clients';
import { Domains } from './Domains';
import { css } from '@emotion/react';
import { RouterProvider, createBrowserRouter } from 'react-router-dom';
import { Alignment, Breadcrumbs, Button, Navbar } from '@blueprintjs/core';
import { Home } from '@blueprintjs/icons';

const queryClient = new QueryClient();

function App() {
  const router = createBrowserRouter([
    {
      path: "/",
      element: <AppGrid />
    }
  ])

  const crumbs = [
    { href: "/", text: "Home", icon: <Home /> }
  ]

  return (
    <>
      <Navbar fixedToTop={true}>
        <Navbar.Group align={Alignment.LEFT}>
          <Navbar.Heading>Penguin</Navbar.Heading>
          <Navbar.Divider />
          <Breadcrumbs items={crumbs} />
        </Navbar.Group>
      </Navbar>
      <div css={css`padding: 82px 25px 25px 25px; width: 100%; height: 100%; max-width: 1280px`}>
        <QueryClientProvider client={queryClient}>
          <RouterProvider router={router} />
        </QueryClientProvider>
      </div>
    </>
  )
}

function AppGrid() {
  const gridStyle = css`
    display: grid;
    width: 100%;
    grid-template-columns: 50% 50%;
    grid-gap: 18px;
  `
  return (
    <div css={gridStyle}>
        <span><Clients /></span>
        <span><Domains /></span>
    </div>
  )
}


export default App
