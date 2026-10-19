package dev.mosaicforge.api;

import jakarta.servlet.FilterChain;
import jakarta.servlet.ServletException;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.util.List;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.env.Environment;
import org.springframework.security.authentication.AuthenticationManager;
import org.springframework.security.authentication.BadCredentialsException;
import org.springframework.security.authentication.UsernamePasswordAuthenticationToken;
import org.springframework.security.config.annotation.web.builders.HttpSecurity;
import org.springframework.security.config.http.SessionCreationPolicy;
import org.springframework.security.core.context.SecurityContextHolder;
import org.springframework.security.web.SecurityFilterChain;
import org.springframework.security.web.authentication.UsernamePasswordAuthenticationFilter;
import org.springframework.web.filter.OncePerRequestFilter;

@Configuration
class Security {
  // No user/password authentication or generated development password for a service-only boundary.
  @Bean
  AuthenticationManager authenticationManager() {
    return authentication -> {
      throw new BadCredentialsException("Service credentials required");
    };
  }

  @Bean
  SecurityFilterChain serviceSecurity(HttpSecurity http, Environment environment) throws Exception {
    String token = environment.getRequiredProperty("mosaicforge.api-token");
    if (!token.matches("[!-~]{32,256}"))
      throw new IllegalArgumentException("Private API token required");
    return http.csrf(csrf -> csrf.disable())
        .requestCache(cache -> cache.disable())
        .sessionManagement(
            session -> session.sessionCreationPolicy(SessionCreationPolicy.STATELESS))
        .authorizeHttpRequests(
            auth -> auth.requestMatchers("/health").permitAll().anyRequest().authenticated())
        .addFilterBefore(
            new OncePerRequestFilter() {
              @Override
              protected void doFilterInternal(
                  HttpServletRequest request, HttpServletResponse response, FilterChain chain)
                  throws ServletException, IOException {
                response.setHeader(
                    "traceparent", Tracing.context(request.getHeader("traceparent")));
                response.setHeader("Cache-Control", "no-store");
                response.setHeader("X-Content-Type-Options", "nosniff");
                if (!("127.0.0.1:" + request.getLocalPort()).equals(request.getHeader("Host"))
                    || request.getHeader("Origin") != null) {
                  Gateway.failure(response, 403, Gateway.UNKNOWN_ID, "forbidden");
                  return;
                }
                if (!request.getRequestURI().equals("/health")) {
                  var provided = request.getHeader("Authorization");
                  if (provided == null
                      || !MessageDigest.isEqual(
                          ("Bearer " + token).getBytes(StandardCharsets.US_ASCII),
                          provided.getBytes(StandardCharsets.UTF_8))) {
                    Gateway.failure(response, 401, Gateway.UNKNOWN_ID, "unauthorized");
                    return;
                  }
                  var context = SecurityContextHolder.createEmptyContext();
                  context.setAuthentication(
                      new UsernamePasswordAuthenticationToken("web-bff", null, List.of()));
                  SecurityContextHolder.setContext(context);
                }
                chain.doFilter(request, response);
              }
            },
            UsernamePasswordAuthenticationFilter.class)
        .build();
  }
}
