
# Contribuindo para MCPRS

Obrigado pelo seu interesse em contribuir para o projeto MCPRS! Este documento fornece orientações para ajudá-lo a contribuir de forma eficaz.

## Código de Conduta

Este projeto adere ao [Código de Conduta do Contributor Covenant](https://www.contributor-covenant.org/version/2/0/code_of_conduct/). Ao participar, você deve respeitar este código.

## Fluxo de Trabalho de Contribuição

1. Faça um fork do repositório no GitHub
2. Clone seu fork: `git clone https://github.com/seu-usuario/mcprs.git`
3. Crie um branch para suas alterações: `git checkout -b feature/sua-feature`
4. Faça suas alterações e adicione testes quando aplicável
5. Execute os testes: `cargo test`
6. Verifique a formatação do código: `cargo fmt --check`
7. Execute o analisador de código: `cargo clippy -- -D warnings`
8. Faça commit das suas alterações: `git commit -m "Descrição clara da alteração"`
9. Envie para seu fork: `git push origin feature/sua-feature`
10. Abra um Pull Request no repositório original

## Diretrizes de Código

- Siga o estilo de código Rust idiomático (use `cargo fmt`)
- Documente todas as funções, structs e traits públicas seguindo as [diretrizes de documentação do Rust](https://doc.rust-lang.org/rustdoc/what-is-rustdoc.html)
- Adicione testes para novas funcionalidades
- Mantenha a cobertura de testes atual ou melhore-a
- Use tipos fortes e evite `unwrap()` em código de produção
- Lide com erros apropriadamente, propagando-os quando necessário

## Reportando Bugs

Use as issues do GitHub para reportar bugs. Inclua:

- Uma descrição clara do problema
- Passos precisos para reproduzi-lo
- Qual comportamento você esperava e o que aconteceu
- Informações sobre ambiente, versões e configurações

## Solicitando Recursos

Novas funcionalidades devem ser discutidas através de issues antes da implementação. Inclua:

- Descrição clara do recurso
- Motivação e casos de uso
- Esboço de como você imagina que a API poderia ser

## Adicionando Novos Agentes

Para contribuir com um novo agente:

1. Crie um novo arquivo `src/agent_nome.rs`
2. Implemente a trait `AIAgent`
3. Adicione testes em `tests/agent_nome_test.rs`
4. Documente todos os métodos públicos
5. Adicione a nova exportação ao `lib.rs`
6. Atualize o README.md com informações sobre o novo agente

## Processo de Release

As versões seguem [Versionamento Semântico](https://semver.org/):

- Versões **major** (x.0.0) para mudanças incompatíveis com versões anteriores
- Versões **minor** (0.x.0) para novas funcionalidades compatíveis
- Versões **patch** (0.0.x) para correções de bugs compatíveis

## Licença

Ao contribuir para este projeto, você concorda que suas contribuições serão licenciadas sob a mesma licença do projeto (MIT).
