import './App.css'
import { Clients } from './Clients';
import { Domains } from './Domains';
import { css } from '@emotion/react';
import { Outlet, useMatches } from 'react-router-dom';
import { Alignment, Breadcrumbs, Navbar } from '@blueprintjs/core';


function App() {
  let matches = useMatches();
  const crumbs = matches
      .filter(m => Boolean((m.handle as any)?.crumb))
      .map(m => (m.handle as any).crumb(m.data));

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
        <Outlet />
      </div>
    </>
  )
}

export function AppGrid() {
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
