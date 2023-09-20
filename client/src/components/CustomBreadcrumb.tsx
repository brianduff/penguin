/// Breadcrumb renderer that uses router dom's <Link /> element instead
/// of <a>.

import { BreadcrumbProps, Classes, Icon } from "@blueprintjs/core";
import { Link } from "react-router-dom";

export function CustomBreadcrumbCurrent(props: BreadcrumbProps) {
  props.current = true;
  return CustomBreadcrumb(props);
}

export function CustomBreadcrumb(props: BreadcrumbProps) {
  const icon = props.icon !== null ? <Icon title={props.iconTitle} icon={props.icon} /> : undefined;
  const classes = [ Classes.BREADCRUMB ];
  if (props.disabled) {
    classes.push(Classes.DISABLED);
  }
  if (props.current) {
    classes.push(Classes.BREADCRUMB_CURRENT);
  }

  return (
    <Link className={classes.join(" ")} to={props.href!}>
      {icon}
      {props.text}
      {props.children}
    </Link>)
}