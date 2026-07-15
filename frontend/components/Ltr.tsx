type Props = React.ComponentPropsWithoutRef<"bdi">;

export function Ltr({ children, ...rest }: Props) {
  return (
    <bdi dir="ltr" {...rest}>
      {children}
    </bdi>
  );
}
