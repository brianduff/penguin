import './App.css'
import { Clients } from './Clients';
import { Domains } from './Domains';
import { css } from '@emotion/react';
import { Outlet, useMatches } from 'react-router-dom';
import { Alignment, Breadcrumbs, Navbar, Switch, Tooltip } from '@blueprintjs/core';
import { CustomBreadcrumb, CustomBreadcrumbCurrent } from './components/CustomBreadcrumb';
import { gridStyle } from './commonstyles';
import { CredentialsContext } from './components/GoogleAuth';
import { useContext } from 'react';
import { useQuery } from 'react-query';
import { getProxyStatus } from './api';
import { ActiveState } from './bindings/ServiceStatus';


function App() {
  let credentials = useContext(CredentialsContext);
  let matches = useMatches();
  const crumbs = matches
      .filter(m => Boolean((m.handle as any)?.crumb))
      .map(m => (m.handle as any).crumb(m.data));

  let { isLoading, data } = useQuery("proxy", () => getProxyStatus());

  return (
    <>
      <Navbar fixedToTop={true}>
        <Navbar.Group align={Alignment.LEFT}>
          <Navbar.Heading>Penguin</Navbar.Heading>
          <Navbar.Divider />
          <Breadcrumbs breadcrumbRenderer={CustomBreadcrumb} currentBreadcrumbRenderer={CustomBreadcrumbCurrent} items={crumbs} />
        </Navbar.Group>
        <Navbar.Group align={Alignment.RIGHT}>
          <span css={css`margin-right: 5px; padding-top: 8px;`}>
              <Switch disabled={isLoading} checked={!isLoading && data?.unwrap().active == ActiveState.ACTIVE}>Proxy enabled</Switch>
          </span>
          <Tooltip minimal={true} content={<span>Signed in as {credentials?.name}</span>}>
            <img src={credentials?.picture} css={css`width: 35px; border-radius: 50%; border: 1px solid`} />
          </Tooltip>
        </Navbar.Group>
      </Navbar>
      <div css={css`padding: 82px 25px 25px 25px; width: 100%; height: 100%; max-width: 1280px`}>
        <Outlet />
      </div>
    </>
  )
}

export function AppGrid() {
  return (
    <div css={gridStyle}>
      <span><Clients /></span>
      <span><Domains /></span>
    </div>
  )
}


export default App
